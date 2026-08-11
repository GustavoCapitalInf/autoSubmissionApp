//! File storage for payload PDFs and seasonality PNGs.
//!
//! Production: Supabase Storage (private bucket, service-role key).
//! Fallback: local disk under the data directory, for single-box installs
//! and development without Supabase credentials.

use std::path::PathBuf;

use anyhow::{anyhow, Result};

pub enum Storage {
    Local { root: PathBuf },
    Supabase {
        base: String,
        service_key: String,
        bucket: String,
        client: reqwest::Client,
    },
}

impl Storage {
    pub async fn init(&self) -> Result<()> {
        match self {
            Storage::Local { root } => {
                std::fs::create_dir_all(root)?;
                Ok(())
            }
            Storage::Supabase { base, service_key, bucket, client } => {
                // Best-effort: try to create the private bucket if it doesn't
                // exist yet. This must not block startup — Supabase's
                // create-bucket call goes through PostgREST internally and
                // has been seen to return spurious errors (e.g. PGRST125)
                // with newer sb_secret_* keys even though the key is valid
                // and object reads/writes work fine. If this fails, log a
                // warning; create the bucket by hand in the dashboard
                // (Storage → New bucket → private) if uploads later 404.
                let resp = client
                    .post(format!("{base}/storage/v1/bucket"))
                    .bearer_auth(service_key)
                    .header("apikey", service_key)
                    .json(&serde_json::json!({ "name": bucket, "public": false }))
                    .send()
                    .await;
                match resp {
                    Ok(r) if r.status().is_success() || r.status() == reqwest::StatusCode::CONFLICT => {}
                    Ok(r) => {
                        let status = r.status();
                        let body = r.text().await.unwrap_or_default();
                        if !body.contains("already exists") {
                            eprintln!(
                                "warning: could not confirm/create Supabase Storage bucket '{bucket}' \
                                 ({status}): {body}. If document uploads fail, create a private bucket \
                                 named '{bucket}' by hand: Supabase dashboard → Storage → New bucket."
                            );
                        }
                    }
                    Err(e) => eprintln!(
                        "warning: could not reach Supabase Storage to create bucket '{bucket}': {e}"
                    ),
                }
                Ok(())
            }
        }
    }

    pub async fn put(&self, key: &str, bytes: Vec<u8>, content_type: &str) -> Result<()> {
        match self {
            Storage::Local { root } => {
                let path = root.join(key);
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(path, bytes)?;
                Ok(())
            }
            Storage::Supabase { base, service_key, bucket, client } => {
                let resp = client
                    .post(format!("{base}/storage/v1/object/{bucket}/{key}"))
                    .bearer_auth(service_key)
                    .header("apikey", service_key)
                    .header("Content-Type", content_type)
                    .header("x-upsert", "true")
                    .body(bytes)
                    .send()
                    .await?;
                if resp.status().is_success() {
                    Ok(())
                } else {
                    let status = resp.status();
                    let body = resp.text().await.unwrap_or_default();
                    Err(anyhow!("uploading {key}: {status} {body}"))
                }
            }
        }
    }

    pub async fn get(&self, key: &str) -> Result<Vec<u8>> {
        match self {
            Storage::Local { root } => Ok(std::fs::read(root.join(key))?),
            Storage::Supabase { base, service_key, bucket, client } => {
                let resp = client
                    .get(format!("{base}/storage/v1/object/{bucket}/{key}"))
                    .bearer_auth(service_key)
                    .header("apikey", service_key)
                    .send()
                    .await?;
                if resp.status().is_success() {
                    Ok(resp.bytes().await?.to_vec())
                } else {
                    Err(anyhow!("fetching {key}: {}", resp.status()))
                }
            }
        }
    }
}
