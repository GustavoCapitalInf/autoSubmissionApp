/// Display-only risk band driving the initials-tile tint.
///
/// Rule taken from the design handoff ("a reasonable rule to implement") —
/// the handoff explicitly asks that the real rule be confirmed with the
/// business before shipping:
///   high  — NSFs >= 5, or FICO < 600, or position >= 3
///   watch — NSFs 1–4, or FICO < 660
///   clean — otherwise
pub fn derive_risk(nsf: i64, fico: Option<i64>, position: i64) -> &'static str {
    let fico = fico.unwrap_or(700);
    if nsf >= 5 || fico < 600 || position >= 3 {
        "high"
    } else if nsf >= 1 || fico < 660 {
        "watch"
    } else {
        "clean"
    }
}
