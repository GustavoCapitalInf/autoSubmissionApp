use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

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
    /// Seasonality-calc rows from the submission sheet (month-by-month table).
    pub season_calc: Option<Value>,
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
    pub content_type: String,
    pub category: Option<String>,
    pub has_file: bool,
}

/// Payload accepted by `POST /api/deals` — the deal record from the source
/// submission sheet. Deliberately forgiving: unknown fields are ignored,
/// numbers may arrive as strings ("650"), and the sheet's own field names
/// (`selectedFunders`, `creditScore`, `timeInBusiness`, `files`, …) are
/// accepted alongside the original API names.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestDeal {
    #[serde(default, deserialize_with = "de_opt_string")]
    pub company: Option<String>,
    /// The submission sheet sends the company under `dealName` (sometimes
    /// alongside `company`); either satisfies the requirement.
    #[serde(default, deserialize_with = "de_opt_string")]
    pub deal_name: Option<String>,
    #[serde(default)]
    pub initials: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string")]
    pub email: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string")]
    pub phone: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string")]
    pub puller: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string")]
    pub re_puller: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string")]
    pub state: Option<String>,
    /// Time in business, e.g. "6 yr 2 mo" (or the sheet's bare "17.5").
    #[serde(default, alias = "timeInBusiness", deserialize_with = "de_opt_string")]
    pub tib: Option<String>,
    #[serde(default = "default_position", deserialize_with = "de_i64_or_1")]
    pub position: i64,
    #[serde(default, deserialize_with = "de_opt_string")]
    pub lead_source: Option<String>,
    /// Credit score.
    #[serde(default, alias = "creditScore", deserialize_with = "de_opt_i64")]
    pub fico: Option<i64>,
    #[serde(default, deserialize_with = "de_opt_string")]
    pub industry: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string")]
    pub revenue: Option<String>,
    /// Average daily balance, e.g. "$11.4k".
    #[serde(default, alias = "avgDailyBalance", deserialize_with = "de_opt_string")]
    pub adb: Option<String>,
    #[serde(default, deserialize_with = "de_opt_string")]
    pub deposits: Option<String>,
    #[serde(default, deserialize_with = "de_i64_or_0")]
    pub nsf: i64,
    /// Requested amount in whole dollars (optional — the sheet doesn't send it).
    #[serde(default, deserialize_with = "de_i64_or_0")]
    pub request: i64,
    /// Matched / suggested lenders (`selectedFunders` on the sheet).
    #[serde(default, alias = "selectedFunders")]
    pub lenders: Vec<String>,
    /// 12 monthly deposit values (optional if a PNG is supplied).
    #[serde(default)]
    pub season: Option<Vec<f64>>,
    /// Month-by-month seasonality-calc rows from the sheet, kept verbatim and
    /// rendered as a table on the desk.
    #[serde(default, alias = "seasonalityCalcRows")]
    pub season_calc: Option<Value>,
    /// Seasonality breakdown PNG, base64.
    #[serde(default)]
    pub seasonality_png: Option<String>,
    #[serde(default, alias = "notes", deserialize_with = "de_opt_string")]
    pub note: Option<String>,
    /// "Santi Notes" block from the source sheet. (`sanitNote` accepted for
    /// compatibility with the handoff's typo.)
    #[serde(
        default,
        alias = "sanitNote",
        alias = "santiNotes",
        deserialize_with = "de_opt_string"
    )]
    pub santi_note: Option<String>,
    /// RFC 3339; defaults to arrival time.
    #[serde(default)]
    pub submitted_at: Option<String>,
    /// Optional override of the derived risk band: clean | watch | high.
    #[serde(default)]
    pub risk: Option<String>,
    /// Attached files (`files` on the sheet, `documents` on the original API).
    #[serde(default, alias = "files")]
    pub documents: Vec<IngestDocument>,
}

impl IngestDeal {
    /// Company display name — `company`, falling back to the sheet's `dealName`.
    pub fn company_name(&self) -> Option<&str> {
        non_empty(self.company.as_deref()).or_else(|| non_empty(self.deal_name.as_deref()))
    }
}

fn non_empty(s: Option<&str>) -> Option<&str> {
    s.map(str::trim).filter(|s| !s.is_empty())
}

fn default_position() -> i64 {
    1
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestDocument {
    pub name: String,
    #[serde(default, deserialize_with = "de_opt_i64")]
    pub pages: Option<i64>,
    #[serde(default, deserialize_with = "de_opt_i64")]
    pub size_bytes: Option<i64>,
    /// File bytes, base64 (a `data:` URI is also accepted). The sheet sends
    /// this as `content`; an empty string means "no file attached".
    #[serde(default, alias = "content")]
    pub content_base64: Option<String>,
    /// Declared MIME type; the server also sniffs the decoded bytes.
    #[serde(default, deserialize_with = "de_opt_string")]
    pub content_type: Option<String>,
    /// e.g. "application", "bank_statements".
    #[serde(default, deserialize_with = "de_opt_string")]
    pub category: Option<String>,
}

/* ── Forgiving deserializers ──────────────────────────────────────────
   The submission sheet serializes nearly everything as strings; these accept
   JSON numbers, numeric strings, or null interchangeably. */

fn value_to_i64(v: &Value) -> Option<i64> {
    match v {
        Value::Number(n) => n.as_i64().or_else(|| n.as_f64().map(|f| f.round() as i64)),
        Value::String(s) => {
            let t: String = s.trim().chars().filter(|c| *c != ',' && *c != '$').collect();
            if t.is_empty() {
                None
            } else {
                t.parse::<i64>()
                    .ok()
                    .or_else(|| t.parse::<f64>().ok().map(|f| f.round() as i64))
            }
        }
        _ => None,
    }
}

fn de_opt_i64<'de, D: Deserializer<'de>>(d: D) -> Result<Option<i64>, D::Error> {
    let v = Option::<Value>::deserialize(d)?;
    Ok(v.as_ref().and_then(value_to_i64))
}

fn de_i64_or_0<'de, D: Deserializer<'de>>(d: D) -> Result<i64, D::Error> {
    Ok(de_opt_i64(d)?.unwrap_or(0))
}

fn de_i64_or_1<'de, D: Deserializer<'de>>(d: D) -> Result<i64, D::Error> {
    Ok(de_opt_i64(d)?.unwrap_or(1))
}

/// String field that may arrive as a number; empty strings become `None`.
fn de_opt_string<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    let v = Option::<Value>::deserialize(d)?;
    Ok(match v {
        Some(Value::String(s)) => {
            let t = s.trim();
            if t.is_empty() { None } else { Some(t.to_string()) }
        }
        Some(Value::Number(n)) => Some(n.to_string()),
        Some(Value::Bool(b)) => Some(b.to_string()),
        _ => None,
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The submission-sheet payload (trimmed sample from the sheet's webhook)
    /// must deserialize without errors and land in the right fields.
    #[test]
    fn accepts_submission_sheet_payload() {
        let payload = r#"{
            "dealName": "Scorch Agency LLC",
            "company": "Scorch Agency LLC",
            "clientId": "229bffb0-69dc-4ee5-b3ed-bcd98402ccd0",
            "clientLink": "https://www.orbit-technology.com/clients/229bffb0",
            "selectedFunders": ["Byzfunder", "Fundworks", "Forward"],
            "puller": "Cristian Piña",
            "rePuller": "",
            "email": "chrisbuehler@scorchagency.com",
            "phone": "3144940696",
            "zohoLink": "https://crm.zoho.com/crm/org683020569",
            "notes": "",
            "revenue": "3",
            "totalMonthlyRevenue": "15000",
            "avgMonthlyRevenue": "44445",
            "avgDailyBalance": "4444",
            "creditScore": "650",
            "experianScore": "",
            "dob": "08/01/1975",
            "timeInBusiness": "17.5",
            "industry": "Advertising / Marketing / Consulting",
            "state": "MO",
            "position": "1",
            "minMonthlyRevenue": "4444",
            "nsf": "5",
            "deposits": "18",
            "leadSource": "Phil - Ferrari",
            "santiNotes": "",
            "seasonalityCalcName": "seasonality_calc.json",
            "seasonalityCalcMime": "application/json",
            "seasonalityCalcContent": "",
            "seasonalityCalcDataUri": "",
            "seasonalityCalcRows": [
                { "month": "March", "year": null, "monthlyDeposits": 180136,
                  "totalMonthlyRevenue": 180136, "depositsPerMonth": 30,
                  "ledger": null, "averageDailyBalance": null, "nsf": 5 },
                { "month": "June", "year": "2026", "monthlyDeposits": 97950,
                  "totalMonthlyRevenue": 97950, "depositsPerMonth": 15,
                  "ledger": null, "averageDailyBalance": null, "nsf": 3 }
            ],
            "files": [
                { "name": "Signed_Application_Christopher_Buehler.pdf", "content": "",
                  "contentType": "application/pdf", "category": "application" },
                { "name": "document92322045.csv", "content": "",
                  "contentType": "application/pdf", "category": "bank_statements" },
                { "name": "document-2.pdf", "contentType": "application/pdf",
                  "category": "bank_statements" }
            ],
            "submittedAt": "2026-08-10T19:29:30.018Z"
        }"#;

        let deal: IngestDeal = serde_json::from_str(payload).expect("payload should deserialize");
        assert_eq!(deal.company_name(), Some("Scorch Agency LLC"));
        assert_eq!(deal.fico, Some(650));
        assert_eq!(deal.nsf, 5);
        assert_eq!(deal.position, 1);
        assert_eq!(deal.request, 0);
        assert_eq!(deal.tib.as_deref(), Some("17.5"));
        assert_eq!(deal.adb.as_deref(), Some("4444"));
        assert_eq!(deal.deposits.as_deref(), Some("18"));
        assert_eq!(deal.lenders.len(), 3);
        assert_eq!(deal.note, None); // empty string → None
        assert_eq!(deal.santi_note, None);
        assert_eq!(deal.lead_source.as_deref(), Some("Phil - Ferrari"));
        let rows = deal.season_calc.as_ref().and_then(Value::as_array).expect("season rows");
        assert_eq!(rows.len(), 2);
        assert_eq!(deal.documents.len(), 3);
        assert_eq!(deal.documents[0].category.as_deref(), Some("application"));
        assert_eq!(deal.documents[0].content_base64.as_deref(), Some(""));
        assert!(deal.documents[2].content_base64.is_none());
    }

    /// The original API field names keep working.
    #[test]
    fn accepts_original_api_payload() {
        let payload = r#"{
            "company": "Brightline Logistics Corp",
            "fico": 702,
            "tib": "9 yr 4 mo",
            "position": 1,
            "nsf": 0,
            "request": 150000,
            "lenders": ["East Harbor", "Newtek"],
            "note": "Clean file.",
            "documents": [
                { "name": "Signed application", "pages": 3, "sizeBytes": 717824 }
            ]
        }"#;
        let deal: IngestDeal = serde_json::from_str(payload).expect("payload should deserialize");
        assert_eq!(deal.company_name(), Some("Brightline Logistics Corp"));
        assert_eq!(deal.fico, Some(702));
        assert_eq!(deal.request, 150000);
        assert_eq!(deal.documents[0].pages, Some(3));
        assert_eq!(deal.documents[0].size_bytes, Some(717_824));
    }
}
