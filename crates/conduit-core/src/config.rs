use std::net::IpAddr;
use std::path::Path;

use serde::{Deserialize, Serialize};

fn env_string(name: &str, target: &mut String) {
    if let Ok(v) = std::env::var(name) {
        if !v.trim().is_empty() {
            *target = v.trim().to_string();
        }
    }
}

fn env_bool(name: &str, target: &mut bool) {
    if let Ok(v) = std::env::var(name) {
        *target = matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes");
    }
}

/// Service configuration. Environment variables always win, so a hosted
/// deployment (container/VM) can be configured entirely through its secret
/// store; `server.toml` in the data directory is the dev-box fallback and is
/// written with freshly generated keys on first run.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    /// `CONDUIT_BIND`
    pub bind: IpAddr,
    /// `CONDUIT_PORT` / `PORT`
    pub port: u16,
    /// `DATABASE_URL` — Supabase Postgres connection string (use the *session*
    /// pooler or direct connection; the transaction pooler breaks prepared
    /// statements).
    pub database_url: String,
    /// `CONDUIT_INGEST_KEY` — required in `X-Api-Key` by the external API to
    /// push deals. Cannot decide deals.
    pub ingest_api_key: String,
    /// `CONDUIT_DESK_KEY` — required by the desk apps for reads, decisions,
    /// files, and events.
    pub desk_api_key: String,
    /// `SEED_DEMO` — seed the design-handoff demo deals when the DB is empty.
    pub seed_demo: bool,
    /// `SUPABASE_URL` — e.g. https://xyzcompany.supabase.co. When set together
    /// with the service key, PDFs/PNGs go to Supabase Storage; otherwise they
    /// are stored on the server's local disk.
    pub supabase_url: String,
    /// `SUPABASE_SERVICE_KEY` — service-role key (server-side secret).
    pub supabase_service_key: String,
    /// `CONDUIT_STORAGE_BUCKET`
    pub storage_bucket: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind: "0.0.0.0".parse().unwrap(),
            port: 4820,
            database_url: String::new(),
            ingest_api_key: uuid::Uuid::new_v4().simple().to_string(),
            desk_api_key: uuid::Uuid::new_v4().simple().to_string(),
            seed_demo: false,
            supabase_url: String::new(),
            supabase_service_key: String::new(),
            storage_bucket: "conduit-files".into(),
        }
    }
}

impl ServerConfig {
    pub fn load(data_dir: Option<&Path>) -> Self {
        let mut cfg = match data_dir.map(|d| d.join("server.toml")) {
            Some(path) if path.exists() => std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| toml::from_str(&s).ok())
                .unwrap_or_default(),
            Some(path) => {
                let cfg = Self::default();
                // Best-effort template for dev boxes; hosted deployments
                // configure via env and may not have a writable home.
                if let Ok(s) = toml::to_string_pretty(&cfg) {
                    let _ = std::fs::write(&path, s);
                }
                cfg
            }
            None => Self::default(),
        };
        env_string("DATABASE_URL", &mut cfg.database_url);
        if let Ok(v) = std::env::var("CONDUIT_BIND") {
            if let Ok(ip) = v.trim().parse() {
                cfg.bind = ip;
            }
        }
        for name in ["CONDUIT_PORT", "PORT"] {
            if let Ok(v) = std::env::var(name) {
                if let Ok(p) = v.trim().parse() {
                    cfg.port = p;
                    break;
                }
            }
        }
        env_string("CONDUIT_INGEST_KEY", &mut cfg.ingest_api_key);
        env_string("CONDUIT_DESK_KEY", &mut cfg.desk_api_key);
        env_bool("SEED_DEMO", &mut cfg.seed_demo);
        env_string("SUPABASE_URL", &mut cfg.supabase_url);
        env_string("SUPABASE_SERVICE_KEY", &mut cfg.supabase_service_key);
        env_string("CONDUIT_STORAGE_BUCKET", &mut cfg.storage_bucket);
        cfg
    }

    pub fn use_supabase_storage(&self) -> bool {
        !self.supabase_url.is_empty() && !self.supabase_service_key.is_empty()
    }
}

/// Desk app configuration — where the hosted service lives and the desk API
/// key. `desk.toml` in the data directory, overridden by `CONDUIT_SERVER_URL`
/// and `CONDUIT_DESK_KEY`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DeskConfig {
    pub server_url: String,
    pub desk_api_key: String,
}

impl Default for DeskConfig {
    fn default() -> Self {
        Self {
            server_url: "http://127.0.0.1:4820".into(),
            desk_api_key: String::new(),
        }
    }
}

impl DeskConfig {
    /// Defaults compiled into the binary. Release builds for a specific
    /// deployment can bake in the server URL and desk key
    /// (`CONDUIT_DEFAULT_SERVER_URL` / `CONDUIT_DEFAULT_DESK_KEY` at build
    /// time — see the Windows installer workflow), so installing the app is
    /// all a reviewer has to do. desk.toml and env vars still override.
    fn compiled_default() -> Self {
        Self {
            server_url: option_env!("CONDUIT_DEFAULT_SERVER_URL")
                .filter(|s| !s.trim().is_empty())
                .unwrap_or("http://127.0.0.1:4820")
                .trim()
                .to_string(),
            desk_api_key: option_env!("CONDUIT_DEFAULT_DESK_KEY")
                .unwrap_or("")
                .trim()
                .to_string(),
        }
    }

    pub fn load(data_dir: Option<&Path>) -> Self {
        let mut cfg = match data_dir.map(|d| d.join("desk.toml")) {
            Some(path) if path.exists() => std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| toml::from_str(&s).ok())
                .unwrap_or_else(Self::compiled_default),
            Some(path) => {
                let cfg = Self::compiled_default();
                if let Ok(s) = toml::to_string_pretty(&cfg) {
                    let _ = std::fs::write(&path, s);
                }
                cfg
            }
            None => Self::compiled_default(),
        };
        // A desk.toml written by an older or unconfigured build may carry the
        // built-in placeholders; treat those as "unset" so baked defaults win.
        let plain = Self::default();
        let baked = Self::compiled_default();
        if cfg.server_url == plain.server_url && baked.server_url != plain.server_url {
            cfg.server_url = baked.server_url.clone();
        }
        if cfg.desk_api_key.is_empty() && !baked.desk_api_key.is_empty() {
            cfg.desk_api_key = baked.desk_api_key.clone();
        }
        env_string("CONDUIT_SERVER_URL", &mut cfg.server_url);
        env_string("CONDUIT_DESK_KEY", &mut cfg.desk_api_key);
        cfg.server_url = cfg.server_url.trim_end_matches('/').to_string();

        // Zero-config dev: pointing at a local server whose server.toml sits
        // in the same data directory, borrow its desk key automatically.
        if cfg.desk_api_key.is_empty() && cfg.is_local() {
            if let Some(dir) = data_dir {
                if let Ok(s) = std::fs::read_to_string(dir.join("server.toml")) {
                    if let Ok(server) = toml::from_str::<ServerConfig>(&s) {
                        cfg.desk_api_key = server.desk_api_key;
                    }
                }
            }
        }
        cfg
    }

    /// True when the configured server is on this machine (dev mode — the
    /// desk will auto-spawn a local server binary if it's down).
    pub fn is_local(&self) -> bool {
        let url = &self.server_url;
        url.contains("://127.") || url.contains("://localhost") || url.contains("://[::1]")
    }
}
