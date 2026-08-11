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
                // Create the private bucket if it doesn't exist yet.
                // Both Authorization and apikey headers are sent so legacy
                // service_role JWTs and new sb_secret_* keys both work.
                let resp = client
                    .post(format!("{base}/storage/v1/bucket"))
                    .bearer_auth(service_key)
                    .header("apikey", service_key)
                    .json(&serde_json::json!({ "name": bucket, "public": false }))
                    .send()
                    .await?;
                let status = resp.status();
                if status.is_success() || status == reqwest::StatusCode::CONFLICT {
                    return Ok(());
                }
                let body = resp.text().await.unwrap_or_default();
                // Supabase reports an existing bucket as a 400 "already exists".
                if body.contains("already exists") {
                    Ok(())
                } else {
                    Err(anyhow!("creating storage bucket '{bucket}': {status} {body}"))
                }
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
