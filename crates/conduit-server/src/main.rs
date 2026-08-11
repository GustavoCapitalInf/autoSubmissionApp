//! Conduit submission service — the hosted, always-on half of the desk.
//!
//! Receives new deals from the external API at any time (no desk needs to be
//! running), persists them to Supabase Postgres (files in Supabase Storage,
//! or local disk when unconfigured), executes approve/reject plus the lender
//! auto-submission job, and streams live events to connected desks over SSE.
//!
//! Auth is two-keyed: the **ingest key** (external API) can only push deals;
//! the **desk key** (reviewer apps) covers reads, decisions, files, events.
//! Configure via env — see `conduit_core::config::ServerConfig`.

mod db;
mod seed;
mod storage;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::{DefaultBodyLimit, Path, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use conduit_core::config::ServerConfig;
use conduit_core::models::IngestDeal;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::convert::Infallible;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use tower_http::cors::CorsLayer;

use crate::storage::Storage;

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    storage: Arc<Storage>,
    tx: broadcast::Sender<String>,
    cfg: Arc<ServerConfig>,
}

struct ApiError(StatusCode, String);

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        let msg = e.to_string();
        let code = if msg.contains("not found") {
            StatusCode::NOT_FOUND
        } else if msg.contains("already decided")
            || msg.contains("must be")
            || msg.contains("not valid")
            || msg.contains("required")
        {
            StatusCode::UNPROCESSABLE_ENTITY
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        };
        ApiError(code, msg)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({ "error": self.1 }))).into_response()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let data_dir = conduit_core::data_dir().ok();
    let cfg = ServerConfig::load(data_dir.as_deref());

    if cfg.database_url.is_empty() {
        anyhow::bail!(
            "DATABASE_URL is not set. Point it at your Supabase Postgres \
             (Project Settings → Database → connection string, session pooler), \
             via environment variable or server.toml in the data directory."
        );
    }

    let storage = if cfg.use_supabase_storage() {
        println!("storage: Supabase bucket '{}'", cfg.storage_bucket);
        Storage::Supabase {
            base: cfg.supabase_url.trim_end_matches('/').to_string(),
            service_key: cfg.supabase_service_key.clone(),
            bucket: cfg.storage_bucket.clone(),
            client: reqwest::Client::new(),
        }
    } else {
        let root = data_dir
            .clone()
            .ok_or_else(|| anyhow::anyhow!(
                "no writable data directory and no Supabase Storage configured — \
                 set SUPABASE_URL and SUPABASE_SERVICE_KEY"
            ))?
            .join("files");
        println!("storage: local disk at {} (set SUPABASE_URL + SUPABASE_SERVICE_KEY for hosted storage)", root.display());
        Storage::Local { root }
    };
    storage.init().await?;

    let pool = db::connect(&cfg.database_url).await?;
    if cfg.seed_demo && seed::seed_if_empty(&pool, &storage).await? {
        println!("seeded demo deals from the design handoff");
    }

    let (tx, _rx) = broadcast::channel::<String>(64);
    let addr = SocketAddr::new(cfg.bind, cfg.port);
    let state = AppState {
        pool,
        storage: Arc::new(storage),
        tx,
        cfg: Arc::new(cfg),
    };

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/api/deals", post(ingest_deal).get(list_deals))
        .route("/api/deals/{id}", get(get_deal))
        .route("/api/deals/{id}/approve", post(approve_deal))
        .route("/api/deals/{id}/reject", post(reject_deal))
        .route("/api/deals/{id}/seasonality", get(seasonality_image))
        .route("/api/documents/{id}/file", get(document_file))
        .route("/api/stats", get(stats))
        .route("/api/events", get(events))
        // Deal payloads carry base64 PDFs — axum's default 2 MB body cap is
        // far too small for a real submission packet.
        .layer(DefaultBodyLimit::max(64 * 1024 * 1024))
        // Desks are WebViews on other origins; endpoints are key-authed.
        .layer(CorsLayer::permissive())
        .with_state(state);

    println!("conduit-server listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

/// Presented key: `X-Api-Key` header, or `?key=` for endpoints loaded
/// directly by the WebView (images, PDFs) where headers can't be set.
fn presented_key<'a>(headers: &'a HeaderMap, query: Option<&'a HashMap<String, String>>) -> Option<&'a str> {
    headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .or_else(|| query.and_then(|q| q.get("key").map(String::as_str)))
}

fn require_key(
    presented: Option<&str>,
    expected: &str,
    role: &str,
) -> Result<(), ApiError> {
    if expected.is_empty() {
        return Err(ApiError(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("server has no {role} key configured"),
        ));
    }
    if presented == Some(expected) {
        Ok(())
    } else {
        Err(ApiError(
            StatusCode::UNAUTHORIZED,
            "missing or invalid X-Api-Key".into(),
        ))
    }
}

fn require_desk(state: &AppState, headers: &HeaderMap, query: Option<&HashMap<String, String>>) -> Result<(), ApiError> {
    require_key(presented_key(headers, query), &state.cfg.desk_api_key, "desk")
}

async fn ingest_deal(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(raw): Json<Value>,
) -> Result<impl IntoResponse, ApiError> {
    require_key(presented_key(&headers, None), &state.cfg.ingest_api_key, "ingest")?;
    // Some webhook relays wrap the record in a top-level `body` key.
    let record = match &raw {
        Value::Object(map) if map.contains_key("body") && map["body"].is_object() => &raw["body"],
        _ => &raw,
    };
    let payload: IngestDeal = serde_json::from_value(record.clone()).map_err(|e| {
        ApiError(
            StatusCode::UNPROCESSABLE_ENTITY,
            format!("deal payload not valid: {e}"),
        )
    })?;
    let deal = db::create_deal(&state.pool, &state.storage, &payload).await?;
    // Connected desks pop the desktop notification on this event.
    let _ = state
        .tx
        .send(json!({ "type": "deal.created", "deal": deal }).to_string());
    Ok((StatusCode::CREATED, Json(deal)))
}

#[derive(Deserialize)]
struct ListQuery {
    status: Option<String>,
}

async fn list_deals(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<ListQuery>,
) -> Result<Json<Value>, ApiError> {
    require_desk(&state, &headers, None)?;
    let mut deals = db::list_deals(&state.pool).await?;
    if let Some(status) = q.status {
        deals.retain(|d| d.status == status);
    }
    Ok(Json(json!({ "deals": deals })))
}

async fn get_deal(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<Value>, ApiError> {
    require_desk(&state, &headers, None)?;
    let deal = db::get_deal(&state.pool, id).await?;
    Ok(Json(json!(deal)))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DecisionBody {
    reviewer: String,
    #[serde(default)]
    reason: Option<String>,
}

async fn approve_deal(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(body): Json<DecisionBody>,
) -> Result<Json<Value>, ApiError> {
    require_desk(&state, &headers, None)?;
    let deal = db::decide_deal(&state.pool, id, true, &body.reviewer, None).await?;
    let _ = state
        .tx
        .send(json!({ "type": "deal.decided", "deal": deal }).to_string());

    // Auto-submission job: send the packet to every matched lender, off the
    // request path. Simulated for now — replace the sleep with the real
    // per-lender submission API calls when the lender integration lands.
    let tx = state.tx.clone();
    let lender_count = deal.lenders.len();
    let company = deal.company.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
        let _ = tx.send(
            json!({
                "type": "submission.completed",
                "dealId": id,
                "company": company,
                "lenderCount": lender_count,
            })
            .to_string(),
        );
    });

    Ok(Json(json!(deal)))
}

async fn reject_deal(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(body): Json<DecisionBody>,
) -> Result<Json<Value>, ApiError> {
    require_desk(&state, &headers, None)?;
    let deal =
        db::decide_deal(&state.pool, id, false, &body.reviewer, body.reason.as_deref()).await?;
    let _ = state
        .tx
        .send(json!({ "type": "deal.decided", "deal": deal }).to_string());
    Ok(Json(json!(deal)))
}

async fn stats(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    require_desk(&state, &headers, None)?;
    let stats = db::stats(&state.pool).await?;
    Ok(Json(json!(stats)))
}

async fn document_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Query(q): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
    require_desk(&state, &headers, Some(&q))?;
    let Some((key, name, content_type)) = db::document_file(&state.pool, id).await? else {
        return Err(ApiError(
            StatusCode::NOT_FOUND,
            "no file stored for this document".into(),
        ));
    };
    let bytes = state.storage.get(&key).await?;
    // Text files (CSV bank exports) are served as text/plain so the WebView
    // previews them inline instead of forcing a download.
    let serve_type = if content_type.starts_with("text/") {
        "text/plain; charset=utf-8".to_string()
    } else {
        content_type
    };
    Ok((
        [
            (header::CONTENT_TYPE, serve_type),
            (
                header::CONTENT_DISPOSITION,
                format!("inline; filename=\"{}\"", name.replace('"', "")),
            ),
        ],
        bytes,
    )
        .into_response())
}

async fn seasonality_image(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Query(q): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
    require_desk(&state, &headers, Some(&q))?;
    let Some(key) = db::seasonality_key(&state.pool, id).await? else {
        return Err(ApiError(
            StatusCode::NOT_FOUND,
            "no seasonality image for this deal".into(),
        ));
    };
    let bytes = state.storage.get(&key).await?;
    Ok(([(header::CONTENT_TYPE, "image/png")], bytes).into_response())
}

async fn events(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>>, ApiError> {
    require_desk(&state, &headers, Some(&q))?;
    let rx = state.tx.subscribe();
    let stream = BroadcastStream::new(rx)
        .filter_map(|msg| msg.ok())
        .map(|msg| Ok(Event::default().data(msg)));
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
