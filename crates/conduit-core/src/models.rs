use serde::{Deserialize, Serialize};

/// Deal record as served to the frontend. Field names mirror the design
/// handoff's deal record exactly (camelCase over the wire).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Deal {
    pub id: i64,
    pub company: String,
    pub initials: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub puller: Option<String>,
    pub re_puller: Option<String>,
    pub state: Option<String>,
    pub tib: Option<String>,
    pub position: i64,
    pub lead_source: Option<String>,
    pub fico: Option<i64>,
    pub industry: Option<String>,
    pub revenue: Option<String>,
    pub adb: Option<String>,
    pub deposits: Option<String>,
    pub nsf: i64,
    pub request: i64,
    pub risk: String,
    pub lenders: Vec<String>,
    /// 12 monthly deposit values, when supplied instead of (or alongside) the PNG.
    pub season: Option<Vec<f64>>,
    pub has_seasonality_image: bool,
    pub note: Option<String>,
    pub santi_note: Option<String>,
    /// RFC 3339; the frontend derives the display and relative forms.
    pub submitted_at: String,
    pub status: String,
    pub decision: Option<String>,
    pub decided_by: Option<String>,
    pub docs: Vec<Document>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub id: i64,
    pub name: String,
    pub pages: i64,
    pub size_bytes: i64,
    pub has_file: bool,
}

/// Payload accepted by `POST /api/deals` — the deal record from the source
/// submission sheet, as listed in the design handoff.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IngestDeal {
    pub company: String,
    #[serde(default)]
    pub initials: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub puller: Option<String>,
    #[serde(default)]
    pub re_puller: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    /// Time in business, e.g. "6 yr 2 mo".
    #[serde(default)]
    pub tib: Option<String>,
    #[serde(default = "default_position")]
    pub position: i64,
    #[serde(default)]
    pub lead_source: Option<String>,
    /// Credit score.
    #[serde(default)]
    pub fico: Option<i64>,
    #[serde(default)]
    pub industry: Option<String>,
    #[serde(default)]
    pub revenue: Option<String>,
    /// Average daily balance, e.g. "$11.4k".
    #[serde(default)]
    pub adb: Option<String>,
    #[serde(default)]
    pub deposits: Option<String>,
    #[serde(default)]
    pub nsf: i64,
    /// Requested amount in whole dollars.
    pub request: i64,
    /// Matched / suggested lenders.
    #[serde(default)]
    pub lenders: Vec<String>,
    /// 12 monthly deposit values (optional if a PNG is supplied).
    #[serde(default)]
    pub season: Option<Vec<f64>>,
    /// Seasonality breakdown PNG, base64.
    #[serde(default)]
    pub seasonality_png: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
    /// "Santi Notes" block from the source sheet. (`sanitNote` accepted for
    /// compatibility with the handoff's typo.)
    #[serde(default, alias = "sanitNote")]
    pub santi_note: Option<String>,
    /// RFC 3339; defaults to arrival time.
    #[serde(default)]
    pub submitted_at: Option<String>,
    /// Optional override of the derived risk band: clean | watch | high.
    #[serde(default)]
    pub risk: Option<String>,
    #[serde(default)]
    pub documents: Vec<IngestDocument>,
}

fn default_position() -> i64 {
    1
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IngestDocument {
    pub name: String,
    #[serde(default)]
    pub pages: Option<i64>,
    #[serde(default)]
    pub size_bytes: Option<i64>,
    /// PDF bytes, base64.
    #[serde(default)]
    pub content_base64: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub in_queue: i64,
    pub auto_submitted: i64,
}

/// Derive the two-letter initials tile text from a company name, e.g.
/// "Brightline Logistics Corp" → "BL", "YNL Fashion Boutique LLC" → "YN".
pub fn derive_initials(company: &str) -> String {
    let words: Vec<&str> = company
        .split_whitespace()
        .filter(|w| w.chars().next().is_some_and(|c| c.is_alphanumeric()))
        .collect();
    if let Some(first) = words.first() {
        // An all-caps acronym leads: take its first two letters (YNL → YN).
        if first.len() >= 2 && first.chars().all(|c| c.is_ascii_uppercase()) {
            return first.chars().take(2).collect();
        }
    }
    let initials: String = words
        .iter()
        .take(2)
        .filter_map(|w| w.chars().next())
        .collect::<String>()
        .to_uppercase();
    if initials.len() >= 2 {
        initials
    } else {
        company.chars().take(2).collect::<String>().to_uppercase()
    }
}
