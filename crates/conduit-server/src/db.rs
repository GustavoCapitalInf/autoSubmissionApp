//! Postgres persistence (Supabase) via sqlx.

use std::collections::HashMap;

use anyhow::{anyhow, Result};
use base64::Engine;
use chrono::{DateTime, Local, Utc};
use conduit_core::models::{derive_initials, Deal, Document, IngestDeal, Stats};
use conduit_core::risk::derive_risk;
use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use sqlx::Row;

use crate::storage::Storage;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS deals (
    id BIGSERIAL PRIMARY KEY,
    company TEXT NOT NULL,
    initials TEXT NOT NULL,
    email TEXT, phone TEXT, puller TEXT, re_puller TEXT,
    state TEXT, tib TEXT,
    position BIGINT NOT NULL DEFAULT 1,
    lead_source TEXT, fico BIGINT, industry TEXT,
    revenue TEXT, adb TEXT, deposits TEXT,
    nsf BIGINT NOT NULL DEFAULT 0,
    request BIGINT NOT NULL DEFAULT 0,
    risk TEXT NOT NULL DEFAULT 'clean',
    lenders JSONB NOT NULL DEFAULT '[]',
    season JSONB,
    seasonality_key TEXT,
    note TEXT, santi_note TEXT,
    submitted_at TIMESTAMPTZ NOT NULL,
    status TEXT NOT NULL DEFAULT 'awaiting',
    decision TEXT, decided_by TEXT, decided_at TIMESTAMPTZ, reject_reason TEXT
);
CREATE TABLE IF NOT EXISTS documents (
    id BIGSERIAL PRIMARY KEY,
    deal_id BIGINT NOT NULL REFERENCES deals(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    pages BIGINT NOT NULL DEFAULT 1,
    size_bytes BIGINT NOT NULL DEFAULT 0,
    storage_key TEXT
);
CREATE INDEX IF NOT EXISTS idx_deals_status ON deals(status);
CREATE INDEX IF NOT EXISTS idx_documents_deal ON documents(deal_id);
";

pub async fn connect(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .map_err(|e| anyhow!("connecting to Postgres (check DATABASE_URL): {e}"))?;
    sqlx::raw_sql(SCHEMA).execute(&pool).await?;
    Ok(pool)
}

const DEAL_COLS: &str = "id, company, initials, email, phone, puller, re_puller, state, tib, \
     position, lead_source, fico, industry, revenue, adb, deposits, nsf, request, risk, \
     lenders, season, seasonality_key, note, santi_note, submitted_at, status, decision, decided_by";

fn deal_from_row(row: &PgRow) -> Deal {
    let lenders: serde_json::Value = row.get("lenders");
    let season: Option<serde_json::Value> = row.get("season");
    let seasonality_key: Option<String> = row.get("seasonality_key");
    let submitted_at: DateTime<Utc> = row.get("submitted_at");
    Deal {
        id: row.get("id"),
        company: row.get("company"),
        initials: row.get("initials"),
        email: row.get("email"),
        phone: row.get("phone"),
        puller: row.get("puller"),
        re_puller: row.get("re_puller"),
        state: row.get("state"),
        tib: row.get("tib"),
        position: row.get("position"),
        lead_source: row.get("lead_source"),
        fico: row.get("fico"),
        industry: row.get("industry"),
        revenue: row.get("revenue"),
        adb: row.get("adb"),
        deposits: row.get("deposits"),
        nsf: row.get("nsf"),
        request: row.get("request"),
        risk: row.get("risk"),
        lenders: serde_json::from_value(lenders).unwrap_or_default(),
        season: season.and_then(|s| serde_json::from_value(s).ok()),
        has_seasonality_image: seasonality_key.is_some(),
        note: row.get("note"),
        santi_note: row.get("santi_note"),
        submitted_at: submitted_at.to_rfc3339(),
        status: row.get("status"),
        decision: row.get("decision"),
        decided_by: row.get("decided_by"),
        docs: Vec::new(),
    }
}

async fn docs_for(pool: &PgPool, deal_ids: &[i64]) -> Result<HashMap<i64, Vec<Document>>> {
    let mut map: HashMap<i64, Vec<Document>> = HashMap::new();
    if deal_ids.is_empty() {
        return Ok(map);
    }
    let rows = sqlx::query(
        "SELECT id, deal_id, name, pages, size_bytes, storage_key FROM documents \
         WHERE deal_id = ANY($1) ORDER BY id",
    )
    .bind(deal_ids)
    .fetch_all(pool)
    .await?;
    for row in rows {
        let deal_id: i64 = row.get("deal_id");
        let storage_key: Option<String> = row.get("storage_key");
        map.entry(deal_id).or_default().push(Document {
            id: row.get("id"),
            name: row.get("name"),
            pages: row.get("pages"),
            size_bytes: row.get("size_bytes"),
            has_file: storage_key.is_some(),
        });
    }
    Ok(map)
}

pub async fn list_deals(pool: &PgPool) -> Result<Vec<Deal>> {
    let rows = sqlx::query(&format!(
        "SELECT {DEAL_COLS} FROM deals ORDER BY submitted_at DESC, id DESC"
    ))
    .fetch_all(pool)
    .await?;
    let mut deals: Vec<Deal> = rows.iter().map(deal_from_row).collect();
    let ids: Vec<i64> = deals.iter().map(|d| d.id).collect();
    let mut docs = docs_for(pool, &ids).await?;
    for deal in &mut deals {
        deal.docs = docs.remove(&deal.id).unwrap_or_default();
    }
    Ok(deals)
}

pub async fn get_deal(pool: &PgPool, id: i64) -> Result<Deal> {
    let row = sqlx::query(&format!("SELECT {DEAL_COLS} FROM deals WHERE id = $1"))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| anyhow!("deal {id} not found"))?;
    let mut deal = deal_from_row(&row);
    deal.docs = docs_for(pool, &[id]).await?.remove(&id).unwrap_or_default();
    Ok(deal)
}

fn sanitize_filename(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() || "-_.".contains(c) { c } else { '_' })
        .collect();
    let trimmed = cleaned.trim_matches(['.', '_']).to_string();
    if trimmed.is_empty() { "document".into() } else { trimmed }
}

pub async fn create_deal(pool: &PgPool, storage: &Storage, deal: &IngestDeal) -> Result<Deal> {
    let submitted_at: DateTime<Utc> = match &deal.submitted_at {
        Some(s) => DateTime::parse_from_rfc3339(s)
            .map_err(|e| anyhow!("submittedAt must be RFC 3339: {e}"))?
            .with_timezone(&Utc),
        None => Utc::now(),
    };
    let initials = deal
        .initials
        .clone()
        .unwrap_or_else(|| derive_initials(&deal.company));
    let risk = match deal.risk.as_deref() {
        Some(r @ ("clean" | "watch" | "high")) => r.to_string(),
        Some(other) => return Err(anyhow!("unknown risk band '{other}'")),
        None => derive_risk(deal.nsf, deal.fico, deal.position).to_string(),
    };

    let deal_id: i64 = sqlx::query_scalar(
        "INSERT INTO deals (company, initials, email, phone, puller, re_puller, state, tib, \
         position, lead_source, fico, industry, revenue, adb, deposits, nsf, request, risk, \
         lenders, season, note, santi_note, submitted_at) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23) \
         RETURNING id",
    )
    .bind(&deal.company)
    .bind(&initials)
    .bind(&deal.email)
    .bind(&deal.phone)
    .bind(&deal.puller)
    .bind(&deal.re_puller)
    .bind(&deal.state)
    .bind(&deal.tib)
    .bind(deal.position)
    .bind(&deal.lead_source)
    .bind(deal.fico)
    .bind(&deal.industry)
    .bind(&deal.revenue)
    .bind(&deal.adb)
    .bind(&deal.deposits)
    .bind(deal.nsf)
    .bind(deal.request)
    .bind(&risk)
    .bind(serde_json::json!(deal.lenders))
    .bind(deal.season.as_ref().map(|s| serde_json::json!(s)))
    .bind(&deal.note)
    .bind(&deal.santi_note)
    .bind(submitted_at)
    .fetch_one(pool)
    .await?;

    if let Some(png_b64) = &deal.seasonality_png {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(png_b64.trim())
            .map_err(|e| anyhow!("seasonalityPng is not valid base64: {e}"))?;
        let key = format!("seasonality/{deal_id}.png");
        storage.put(&key, bytes, "image/png").await?;
        sqlx::query("UPDATE deals SET seasonality_key = $1 WHERE id = $2")
            .bind(&key)
            .bind(deal_id)
            .execute(pool)
            .await?;
    }

    for (i, doc) in deal.documents.iter().enumerate() {
        let mut storage_key: Option<String> = None;
        let mut size = doc.size_bytes.unwrap_or(0);
        if let Some(b64) = &doc.content_base64 {
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(b64.trim())
                .map_err(|e| anyhow!("document '{}' is not valid base64: {e}", doc.name))?;
            size = bytes.len() as i64;
            let key = format!("documents/{deal_id}/{i}-{}.pdf", sanitize_filename(&doc.name));
            storage.put(&key, bytes, "application/pdf").await?;
            storage_key = Some(key);
        }
        sqlx::query(
            "INSERT INTO documents (deal_id, name, pages, size_bytes, storage_key) \
             VALUES ($1,$2,$3,$4,$5)",
        )
        .bind(deal_id)
        .bind(&doc.name)
        .bind(doc.pages.unwrap_or(1))
        .bind(size)
        .bind(&storage_key)
        .execute(pool)
        .await?;
    }

    get_deal(pool, deal_id).await
}

/// Approve or reject atomically — the `status = 'awaiting'` guard means two
/// reviewers on different machines can't double-decide a deal.
pub async fn decide_deal(
    pool: &PgPool,
    id: i64,
    approve: bool,
    reviewer: &str,
    reason: Option<&str>,
) -> Result<Deal> {
    let deal = get_deal(pool, id).await?;
    let now = Utc::now();
    let time = now.with_timezone(&Local).format("%-I:%M %p");
    let (status, stamp) = if approve {
        (
            "approved",
            format!("Submitted by {reviewer} · {time} · {} lenders", deal.lenders.len()),
        )
    } else {
        let reason_suffix = reason
            .map(str::trim)
            .filter(|r| !r.is_empty())
            .map(|r| format!(" · {r}"))
            .unwrap_or_default();
        ("rejected", format!("Rejected by {reviewer} · {time}{reason_suffix}"))
    };
    let updated = sqlx::query(
        "UPDATE deals SET status = $1, decision = $2, decided_by = $3, decided_at = $4, \
         reject_reason = $5 WHERE id = $6 AND status = 'awaiting'",
    )
    .bind(status)
    .bind(&stamp)
    .bind(reviewer)
    .bind(now)
    .bind(reason)
    .bind(id)
    .execute(pool)
    .await?;
    if updated.rows_affected() == 0 {
        return Err(anyhow!("deal {id} was already decided ({})", deal.status));
    }
    get_deal(pool, id).await
}

pub async fn stats(pool: &PgPool) -> Result<Stats> {
    let in_queue: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM deals WHERE status = 'awaiting'")
            .fetch_one(pool)
            .await?;
    let auto_submitted: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM deals WHERE status = 'approved'")
            .fetch_one(pool)
            .await?;
    Ok(Stats { in_queue, auto_submitted })
}

pub async fn deal_count(pool: &PgPool) -> Result<i64> {
    Ok(sqlx::query_scalar("SELECT COUNT(*) FROM deals")
        .fetch_one(pool)
        .await?)
}

pub async fn document_file(pool: &PgPool, doc_id: i64) -> Result<Option<(String, String)>> {
    let row = sqlx::query("SELECT storage_key, name FROM documents WHERE id = $1")
        .bind(doc_id)
        .fetch_optional(pool)
        .await?;
    Ok(row.and_then(|r| {
        let key: Option<String> = r.get("storage_key");
        let name: String = r.get("name");
        key.map(|k| (k, name))
    }))
}

pub async fn seasonality_key(pool: &PgPool, deal_id: i64) -> Result<Option<String>> {
    let row = sqlx::query("SELECT seasonality_key FROM deals WHERE id = $1")
        .bind(deal_id)
        .fetch_optional(pool)
        .await?;
    Ok(row.and_then(|r| r.get("seasonality_key")))
}
