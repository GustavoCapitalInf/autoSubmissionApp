//! Shared core for the Conduit submission desk: config, models, database,
//! demo seed, and the placeholder-PDF generator.

pub mod config;
pub mod models;
pub mod pdf;
pub mod risk;

use std::path::PathBuf;

/// Application data directory (database, documents, seasonality images, config).
pub fn data_dir() -> anyhow::Result<PathBuf> {
    let dirs = directories::ProjectDirs::from("com", "CapitalInfusion", "Conduit")
        .ok_or_else(|| anyhow::anyhow!("could not resolve a home directory"))?;
    let dir = dirs.data_dir().to_path_buf();
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}
