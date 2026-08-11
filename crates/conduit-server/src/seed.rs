//! Demo seed — the nine placeholder deals from the design handoff prototype.
//! Off by default in production; enable with `SEED_DEMO=true` (or
//! `seed_demo = true` in server.toml) to fill an empty database for demos.
//! The reviewer is spelled **Santi** (the handoff's "Sanit" was a typo).

use anyhow::Result;
use chrono::{DateTime, Days, Duration, Local, Utc};
use conduit_core::pdf::placeholder_pdf;
use sqlx::PgPool;

use crate::storage::Storage;

struct SeedDeal {
    company: &'static str,
    initials: &'static str,
    industry: &'static str,
    state: &'static str,
    tib: &'static str,
    position: i64,
    request: i64,
    fico: i64,
    adb: &'static str,
    nsf: i64,
    email: &'static str,
    phone: &'static str,
    puller: &'static str,
    re_puller: &'static str,
    lead_source: &'static str,
    revenue: &'static str,
    deposits: &'static str,
    risk: &'static str,
    lenders: &'static [&'static str],
    season: [f64; 12],
    note: &'static str,
    santi_note: &'static str,
    /// (status, decision stamp, decided by, reject reason)
    decision: Option<(&'static str, &'static str, &'static str, Option<&'static str>)>,
    /// (name, pages, authored display size in bytes)
    docs: &'static [(&'static str, i64, i64)],
}

const KB: i64 = 1024;
const MB: i64 = 1024 * 1024;

pub async fn seed_if_empty(pool: &PgPool, storage: &Storage) -> Result<bool> {
    if crate::db::deal_count(pool).await? > 0 {
        return Ok(false);
    }

    let now = Local::now();
    let minutes_ago = |m: i64| (now - Duration::minutes(m)).with_timezone(&Utc);
    let today_at = |h: u32, m: u32| {
        let t = now
            .date_naive()
            .and_hms_opt(h, m, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap();
        // Keep seed times in the past even when the desk starts early in the day.
        if t >= now { now - Duration::minutes(90) } else { t }.with_timezone(&Utc)
    };
    let yesterday_at = |h: u32, m: u32| {
        (now.date_naive() - Days::new(1))
            .and_hms_opt(h, m, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc)
    };

    let deals: Vec<(SeedDeal, DateTime<Utc>)> = vec![
        (
            SeedDeal {
                company: "YNL Fashion Boutique LLC", initials: "YN",
                industry: "Distributor / Wholesale", state: "IL", tib: "6 yr 2 mo", position: 2,
                request: 74_000, fico: 625, adb: "$11.4k", nsf: 3,
                email: "greenbutterfly20@yahoo.com", phone: "(312) 555-0148",
                puller: "Jonathan Montpeirous", re_puller: "None",
                lead_source: "2023 Mailer — All Avocado — A", revenue: "$84,200 / mo",
                deposits: "22 / mo", risk: "watch",
                lenders: &["East Harbor", "Smartbiz", "LoanBud", "Newtek", "AFB", "CashFlo IT", "Rapid", "Mayfair", "Merit Equipment"],
                season: [38.0, 44.0, 41.0, 52.0, 60.0, 71.0, 66.0, 58.0, 49.0, 55.0, 63.0, 80.0],
                note: "Bank statements complete for 4 months. Landlord verification still outstanding.",
                santi_note: "Two NSFs cluster in the same week — likely a payroll timing issue, not distress. Worth a call before we send.",
                decision: None,
                docs: &[
                    ("Bank statements — Apr–Jul 2026", 4, 2 * MB + 105 * KB),
                    ("Signed application", 3, 640 * KB),
                    ("Driver's license", 1, 310 * KB),
                    ("Voided check", 1, 122 * KB),
                ],
            },
            minutes_ago(4),
        ),
        (
            SeedDeal {
                company: "Brightline Logistics Corp", initials: "BL",
                industry: "Trucking / Freight", state: "TX", tib: "9 yr 4 mo", position: 1,
                request: 150_000, fico: 702, adb: "$26.8k", nsf: 0,
                email: "ap@brightlinelog.com", phone: "(214) 555-0192",
                puller: "Marisol Vega", re_puller: "None",
                lead_source: "Inbound — Organic search", revenue: "$148,900 / mo",
                deposits: "41 / mo", risk: "clean",
                lenders: &["East Harbor", "Newtek", "Rapid", "Mayfair", "Forward Line", "Merit Equipment"],
                season: [62.0, 58.0, 64.0, 70.0, 75.0, 79.0, 84.0, 88.0, 80.0, 76.0, 82.0, 90.0],
                note: "Clean file. Three trucks financed elsewhere, both positions disclosed.",
                santi_note: "First position, 700+ FICO — send to the full list, no call needed.",
                decision: None,
                docs: &[
                    ("Bank statements — Apr–Jul 2026", 4, 2 * MB + 614 * KB),
                    ("Signed application", 3, 701 * KB),
                    ("Articles of incorporation", 6, MB + 410 * KB),
                    ("Driver's license", 1, 288 * KB),
                    ("Existing advance payoff letter", 2, 410 * KB),
                ],
            },
            minutes_ago(11),
        ),
        (
            SeedDeal {
                company: "Cedar & Co Restaurant Group", initials: "CC",
                industry: "Food Service", state: "NY", tib: "3 yr 1 mo", position: 3,
                request: 48_000, fico: 588, adb: "$4.1k", nsf: 7,
                email: "ops@cedarandco.nyc", phone: "N/A",
                puller: "Jonathan Montpeirous", re_puller: "D. Okoye",
                lead_source: "2023 Mailer — All Avocado — B", revenue: "$61,400 / mo",
                deposits: "58 / mo", risk: "high",
                lenders: &["CashFlo IT", "LoanBud", "Rapid"],
                season: [70.0, 66.0, 52.0, 44.0, 38.0, 30.0, 28.0, 33.0, 41.0, 48.0, 57.0, 64.0],
                note: "Third position requested. Two existing advances still amortizing.",
                santi_note: "Seven NSFs and a declining deposit curve. I'd decline unless they can show a July turnaround.",
                decision: None,
                docs: &[
                    ("Bank statements — Apr–Jul 2026", 4, MB + 819 * KB),
                    ("Signed application", 3, 622 * KB),
                    ("Advance #1 contract", 11, 3 * MB + 205 * KB),
                    ("Advance #2 contract", 9, 2 * MB + 717 * KB),
                ],
            },
            minutes_ago(18),
        ),
        (
            SeedDeal {
                company: "Halcyon Dental Partners", initials: "HD",
                industry: "Healthcare", state: "FL", tib: "12 yr 8 mo", position: 1,
                request: 220_000, fico: 741, adb: "$58.9k", nsf: 0,
                email: "finance@halcyondental.com", phone: "(305) 555-0177",
                puller: "Marisol Vega", re_puller: "None",
                lead_source: "Referral — Partner broker", revenue: "$212,000 / mo",
                deposits: "31 / mo", risk: "clean",
                lenders: &["East Harbor", "Newtek", "AFB", "Mayfair", "Forward Line", "Smartbiz", "Merit Equipment"],
                season: [74.0, 78.0, 80.0, 77.0, 82.0, 86.0, 84.0, 88.0, 91.0, 89.0, 93.0, 96.0],
                note: "Practice expansion — third operatory build-out. Equipment quotes attached.",
                santi_note: "Best file on the desk today. Price it aggressively, Careem.",
                decision: None,
                docs: &[
                    ("Bank statements — Apr–Jul 2026", 4, 3 * MB),
                    ("Signed application", 3, 655 * KB),
                    ("Equipment quote — Henry Schein", 5, MB + 102 * KB),
                    ("Practice P&L 2025", 8, 900 * KB),
                    ("Driver's license", 1, 295 * KB),
                    ("Voided check", 1, 118 * KB),
                ],
            },
            minutes_ago(26),
        ),
        (
            SeedDeal {
                company: "Pike Street Auto Body", initials: "PS",
                industry: "Auto Repair", state: "WA", tib: "5 yr 7 mo", position: 2,
                request: 60_000, fico: 649, adb: "$8.9k", nsf: 2,
                email: "billing@pikestbody.com", phone: "(206) 555-0125",
                puller: "Devon Reyes", re_puller: "None",
                lead_source: "2023 Mailer — All Avocado — A", revenue: "$52,300 / mo",
                deposits: "19 / mo", risk: "watch",
                lenders: &["LoanBud", "CashFlo IT", "Rapid", "Smartbiz", "AFB"],
                season: [48.0, 52.0, 55.0, 59.0, 62.0, 58.0, 54.0, 57.0, 61.0, 64.0, 60.0, 66.0],
                note: "Insurance receivables make up 60% of deposits. Aging report requested.",
                santi_note: "Second position is fine here. Hold Rapid back — they burned this merchant in 2024.",
                decision: None,
                docs: &[
                    ("Bank statements — Apr–Jul 2026", 4, MB + 921 * KB),
                    ("Signed application", 3, 610 * KB),
                    ("Insurance receivables aging", 3, 480 * KB),
                ],
            },
            minutes_ago(41),
        ),
        (
            SeedDeal {
                company: "Vantage Modular Build", initials: "VM",
                industry: "Construction", state: "GA", tib: "4 yr 3 mo", position: 4,
                request: 95_000, fico: 611, adb: "$9.2k", nsf: 5,
                email: "accounts@vantagemodular.com", phone: "(404) 555-0163",
                puller: "Devon Reyes", re_puller: "J. Montpeirous",
                lead_source: "Inbound — Paid social", revenue: "$96,700 / mo",
                deposits: "27 / mo", risk: "high",
                lenders: &["CashFlo IT", "Rapid", "LoanBud"],
                season: [55.0, 60.0, 68.0, 72.0, 66.0, 58.0, 50.0, 46.0, 52.0, 57.0, 49.0, 44.0],
                note: "Fourth position. Two liens filed in Q1, both satisfied per merchant.",
                santi_note: "Fourth position at 611 is a stretch. Careem, your call — I lean decline.",
                decision: None,
                docs: &[
                    ("Bank statements — Apr–Jul 2026", 4, 2 * MB + 205 * KB),
                    ("Signed application", 3, 634 * KB),
                    ("Lien release — Q1", 2, 360 * KB),
                    ("Advance #3 contract", 10, 2 * MB + 922 * KB),
                ],
            },
            minutes_ago(58),
        ),
        (
            SeedDeal {
                company: "Northgate Print & Signage", initials: "NP",
                industry: "Printing", state: "OH", tib: "7 yr 5 mo", position: 1,
                request: 80_000, fico: 688, adb: "$18.2k", nsf: 1,
                email: "ar@northgateprint.com", phone: "(614) 555-0139",
                puller: "Marisol Vega", re_puller: "None",
                lead_source: "Referral — Partner broker", revenue: "$91,500 / mo",
                deposits: "24 / mo", risk: "clean",
                lenders: &["East Harbor", "Newtek", "Smartbiz", "Mayfair", "AFB", "Forward Line"],
                season: [60.0, 63.0, 67.0, 70.0, 72.0, 68.0, 74.0, 77.0, 75.0, 79.0, 81.0, 84.0],
                note: "Clean four months. Existing equipment lease disclosed.",
                santi_note: "No concerns — Careem sent it same morning.",
                decision: Some(("approved", "Submitted by Careem · 9:31 AM · 6 lenders", "Careem", None)),
                docs: &[
                    ("Bank statements — Apr–Jul 2026", 4, 2 * MB + 410 * KB),
                    ("Signed application", 3, 618 * KB),
                    ("Equipment lease schedule", 4, 720 * KB),
                ],
            },
            today_at(9, 12),
        ),
        (
            SeedDeal {
                company: "Silver Lake Fitness Co", initials: "SL",
                industry: "Fitness / Gyms", state: "CA", tib: "2 yr 4 mo", position: 3,
                request: 55_000, fico: 574, adb: "$3.4k", nsf: 9,
                email: "owner@silverlakefit.com", phone: "(323) 555-0108",
                puller: "Devon Reyes", re_puller: "None",
                lead_source: "Inbound — Paid social", revenue: "$44,800 / mo",
                deposits: "36 / mo", risk: "high",
                lenders: &["CashFlo IT", "Rapid"],
                season: [58.0, 54.0, 49.0, 44.0, 40.0, 36.0, 33.0, 30.0, 28.0, 31.0, 29.0, 26.0],
                note: "Membership churn evident across the last two statements.",
                santi_note: "Nine NSFs and a falling deposit trend. Declined — revisit in six months.",
                decision: Some(("rejected", "Rejected by Santi · 5:02 PM · NSF volume", "Santi", Some("NSF volume"))),
                docs: &[
                    ("Bank statements — Apr–Jul 2026", 4, MB + 717 * KB),
                    ("Signed application", 3, 605 * KB),
                ],
            },
            yesterday_at(16, 48),
        ),
        (
            SeedDeal {
                company: "Harbor Point Marine Supply", initials: "HP",
                industry: "Retail / Marine", state: "ME", tib: "11 yr 1 mo", position: 2,
                request: 130_000, fico: 715, adb: "$34.6k", nsf: 0,
                email: "finance@harborpointmarine.com", phone: "(207) 555-0154",
                puller: "Jonathan Montpeirous", re_puller: "None",
                lead_source: "2023 Mailer — All Avocado — A", revenue: "$168,300 / mo",
                deposits: "29 / mo", risk: "clean",
                lenders: &["East Harbor", "Newtek", "AFB", "Mayfair", "Smartbiz", "Forward Line", "Merit Equipment"],
                season: [40.0, 46.0, 58.0, 72.0, 86.0, 94.0, 97.0, 90.0, 74.0, 60.0, 48.0, 42.0],
                note: "Strong seasonal peak May–Aug, consistent with marine retail.",
                santi_note: "Seasonality is textbook here — priced off summer averages, not trailing three.",
                decision: Some(("approved", "Submitted by Santi · 1:44 PM · 7 lenders", "Santi", None)),
                docs: &[
                    ("Bank statements — Apr–Jul 2026", 4, 2 * MB + 819 * KB),
                    ("Signed application", 3, 640 * KB),
                    ("Inventory schedule", 5, 980 * KB),
                    ("Voided check", 1, 120 * KB),
                ],
            },
            yesterday_at(13, 20),
        ),
    ];

    for (deal, submitted_at) in &deals {
        let (status, decision, decided_by, reject_reason) = match deal.decision {
            Some((s, d, b, r)) => (s, Some(d), Some(b), r),
            None => ("awaiting", None, None, None),
        };
        let deal_id: i64 = sqlx::query_scalar(
            "INSERT INTO deals (company, initials, email, phone, puller, re_puller, state, tib, \
             position, lead_source, fico, industry, revenue, adb, deposits, nsf, request, risk, \
             lenders, season, note, santi_note, submitted_at, status, decision, decided_by, reject_reason) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24,$25,$26,$27) \
             RETURNING id",
        )
        .bind(deal.company)
        .bind(deal.initials)
        .bind(deal.email)
        .bind(deal.phone)
        .bind(deal.puller)
        .bind(deal.re_puller)
        .bind(deal.state)
        .bind(deal.tib)
        .bind(deal.position)
        .bind(deal.lead_source)
        .bind(deal.fico)
        .bind(deal.industry)
        .bind(deal.revenue)
        .bind(deal.adb)
        .bind(deal.deposits)
        .bind(deal.nsf)
        .bind(deal.request)
        .bind(deal.risk)
        .bind(serde_json::json!(deal.lenders))
        .bind(serde_json::json!(deal.season))
        .bind(deal.note)
        .bind(deal.santi_note)
        .bind(submitted_at)
        .bind(status)
        .bind(decision)
        .bind(decided_by)
        .bind(reject_reason)
        .fetch_one(pool)
        .await?;

        for (i, (name, pages, size)) in deal.docs.iter().enumerate() {
            let key = format!("documents/{deal_id}/{i}-seed.pdf");
            // Demo filler only — a storage hiccup here must not take down the
            // whole server. Record the document without a file rather than
            // aborting startup; the desk shows it, "View" just won't work.
            let stored_key = match storage
                .put(&key, placeholder_pdf(name, *pages as usize), "application/pdf")
                .await
            {
                Ok(()) => Some(key),
                Err(e) => {
                    eprintln!("warning: seed upload for '{name}' failed, skipping file: {e}");
                    None
                }
            };
            sqlx::query(
                "INSERT INTO documents (deal_id, name, pages, size_bytes, storage_key) \
                 VALUES ($1,$2,$3,$4,$5)",
            )
            .bind(deal_id)
            .bind(name)
            .bind(pages)
            .bind(size)
            .bind(&stored_key)
            .execute(pool)
            .await?;
        }
    }

    Ok(true)
}
