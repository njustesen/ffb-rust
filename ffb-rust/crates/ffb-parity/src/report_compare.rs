//! Report-level comparison: Java's `ReportList` stream against Rust's `game.report_list` stream.
//!
//! The per-step state hash says whether the two engines agree on the GAME; this says whether they
//! agree on what they REPORTED about it. Reports are the 1:1 port of Java's `addReport(...)`, so a
//! difference here is a Rust fidelity gap in the report layer even when the hash is green (a
//! missing `ReportReRoll`, a modifier list serialised differently, a report added on one side only).
//!
//! Both files are one JSON object per line with Java's camelCase keys plus a harness `i` (the
//! parity step index the report was produced under). Java attributes a report to the step whose
//! model sync carried it, Rust to the `apply` that appended it, so `i` can legitimately differ by
//! one at a step boundary — the comparison is therefore over the FLATTENED sequence, and `i` is
//! carried only for the diagnostic.
//!
//! Advisory by design: the result is printed next to the parity verdict and does not change it.
use crate::log_format::{java_reports_path_for, rust_reports_path_for};

#[derive(Debug)]
pub struct ReportCompare {
    pub identical: bool,
    pub java_count: usize,
    pub rust_count: usize,
    /// Position in the flattened sequence of the first difference (or the shorter length).
    pub divergence: usize,
    pub java_line: Option<serde_json::Value>,
    pub rust_line: Option<serde_json::Value>,
    /// `ids` when the reportId sequences already differ, `payload` when only the JSON bodies do.
    pub kind: &'static str,
}

fn read(path: &str) -> Option<Vec<serde_json::Value>> {
    let text = std::fs::read_to_string(path).ok()?;
    Some(text.lines().filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .collect())
}

/// Bring the two harnesses' identifiers to one spelling. Java's parity teams are
/// `team<Race>Parity[16|20|25](Home|Away)` with players `...Home14`; Rust's are `home_<roster>` with
/// players `home_14`; coaches are "Home" / "Coach_home". Everything becomes `home` / `home_14`.
/// Anything else is returned unchanged.
pub fn normalise_id(s: &str) -> String {
    fn side_of(word: &str) -> Option<&'static str> {
        match word { "Home" | "home" => Some("home"), "Away" | "away" => Some("away"), _ => None }
    }
    if let Some(rest) = s.strip_prefix("team") {
        // Java: team<Race>Parity[16|20|25]<Home|Away>[<nr>]
        if let Some(idx) = rest.rfind("Home").or_else(|| rest.rfind("Away")) {
            let (_, tail) = rest.split_at(idx);
            let side = side_of(&tail[..4]).unwrap();
            let nr = &tail[4..];
            if nr.is_empty() { return side.to_string(); }
            if let Ok(n) = nr.parse::<u32>() { return format!("{side}_{n:02}"); }
        }
    }
    for side in ["home", "away"] {
        if let Some(rest) = s.strip_prefix(&format!("{side}_")) {
            if let Ok(n) = rest.parse::<u32>() { return format!("{side}_{n:02}"); }
            if !rest.is_empty() && rest.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.') {
                return side.to_string();
            }
        }
    }
    match s { "Home" | "Coach_home" => "home".to_string(), "Away" | "Coach_away" => "away".to_string(), _ => s.to_string() }
}

/// Drop the harness index, `null` fields and empty arrays so "absent", "null" and "[]" compare
/// equal (Java omits unset optionals and some empty arrays, Rust serialises them), rewrite
/// `{x,y}` coordinates as Java's `[x,y]`, and normalise every identifier string.
pub fn normalise(v: &serde_json::Value) -> serde_json::Value {
    match v {
        serde_json::Value::Object(m) => {
            if m.len() == 2 && m.contains_key("x") && m.contains_key("y") {
                return serde_json::Value::Array(vec![m["x"].clone(), m["y"].clone()]);
            }
            serde_json::Value::Object(
                m.iter()
                    .filter(|(k, val)| k.as_str() != "i" && !val.is_null()
                        && !matches!(val, serde_json::Value::Array(a) if a.is_empty()))
                    .map(|(k, val)| {
                        let mut v = normalise(val);
                        if MODIFIER_KEYS.contains(&k.as_str()) {
                            if let serde_json::Value::Array(a) = &mut v {
                                a.sort_by(|x, y| x.to_string().cmp(&y.to_string()));
                            }
                        }
                        (k.clone(), v)
                    })
                    .collect(),
            )
        }
        serde_json::Value::Array(a) => serde_json::Value::Array(a.iter().map(normalise).collect()),
        serde_json::Value::String(s) => serde_json::Value::String(normalise_id(s)),
        other => other.clone(),
    }
}

/// Java builds its modifier collections with `Collectors.toSet()` (`InjuryModifierFactory
/// .getInjuryModifiers`), so the ORDER in `injuryModifiers` / `armorModifiers` / `rollModifiers` is
/// a `HashSet` artefact, not information. Sort those arrays on both sides before comparing.
const MODIFIER_KEYS: [&str; 5] =
    ["armorModifiers", "injuryModifiers", "casualtyModifiers", "rollModifiers", "passModifiers"];

fn report_id(v: &serde_json::Value) -> &str {
    v.get("reportId").and_then(|x| x.as_str()).unwrap_or("")
}

/// `None` when either side has no report file (an older Java cache, a crashed game): the seed is
/// then SKIPPED for the report verdict rather than counted either way.
pub fn compare_reports(seed: u64, edition: &str, home: &str, away: &str) -> Option<ReportCompare> {
    let java = read(&java_reports_path_for(seed, edition, home, away))?;
    let rust = read(&rust_reports_path_for(seed, edition, home, away))?;
    Some(compare_sequences(&java, &rust))
}

pub fn compare_sequences(java: &[serde_json::Value], rust: &[serde_json::Value]) -> ReportCompare {
    let n = java.len().min(rust.len());
    for k in 0..n {
        if report_id(&java[k]) != report_id(&rust[k]) {
            return ReportCompare {
                identical: false, java_count: java.len(), rust_count: rust.len(), divergence: k,
                java_line: Some(java[k].clone()), rust_line: Some(rust[k].clone()), kind: "ids",
            };
        }
    }
    if java.len() != rust.len() {
        return ReportCompare {
            identical: false, java_count: java.len(), rust_count: rust.len(), divergence: n,
            java_line: java.get(n).cloned(), rust_line: rust.get(n).cloned(), kind: "ids",
        };
    }
    for k in 0..n {
        if normalise(&java[k]) != normalise(&rust[k]) {
            return ReportCompare {
                identical: false, java_count: java.len(), rust_count: rust.len(), divergence: k,
                java_line: Some(java[k].clone()), rust_line: Some(rust[k].clone()), kind: "payload",
            };
        }
    }
    ReportCompare {
        identical: true, java_count: java.len(), rust_count: rust.len(), divergence: n,
        java_line: None, rust_line: None, kind: "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn identical_streams_match_and_null_equals_absent() {
        let j = vec![json!({"reportId":"dodgeRoll","roll":4,"i":3}), json!({"reportId":"skillUse","skill":"Dodge","used":true})];
        let r = vec![json!({"reportId":"dodgeRoll","roll":4,"statBasedRollModifier":null,"i":4}), json!({"reportId":"skillUse","skill":"Dodge","used":true})];
        let c = compare_sequences(&j, &r);
        assert!(c.identical, "{c:?}");
    }

    #[test]
    fn identifiers_from_both_harnesses_normalise_to_one_spelling() {
        assert_eq!(normalise_id("teamLinemanParityHome14"), "home_14");
        assert_eq!(normalise_id("teamGoblinParity25Away3"), "away_03");
        assert_eq!(normalise_id("teamHumanParity25Home"), "home");
        assert_eq!(normalise_id("home_14"), "home_14");
        assert_eq!(normalise_id("away_3"), "away_03");
        assert_eq!(normalise_id("home_human.lrb6"), "home");
        assert_eq!(normalise_id("Coach_home"), "home");
        assert_eq!(normalise_id("Home"), "home");
        assert_eq!(normalise_id("Bone Head"), "Bone Head");
        let j = json!({"reportId":"kickoffScatter","ballCoordinateEnd":[22,7]});
        let r = json!({"reportId":"kickoffScatter","ballCoordinateEnd":{"x":22,"y":7}});
        assert_eq!(normalise(&j), normalise(&r));
    }

    #[test]
    fn modifier_lists_compare_as_sets() {
        // Java's modifier collections come out of a HashSet, so their order means nothing.
        let j = vec![json!({"reportId":"injury","injuryModifiers":["Stunty","Mighty Blow"]})];
        let r = vec![json!({"reportId":"injury","injuryModifiers":["Mighty Blow","Stunty"]})];
        assert!(compare_sequences(&j, &r).identical);
        // A genuinely different set is still a difference.
        let r2 = vec![json!({"reportId":"injury","injuryModifiers":["Mighty Blow"]})];
        assert!(!compare_sequences(&j, &r2).identical);
    }

    #[test]
    fn id_sequence_difference_is_reported_first() {
        let j = vec![json!({"reportId":"dodgeRoll"}), json!({"reportId":"reRoll"})];
        let r = vec![json!({"reportId":"dodgeRoll"})];
        let c = compare_sequences(&j, &r);
        assert!(!c.identical);
        assert_eq!(c.kind, "ids");
        assert_eq!(c.divergence, 1);
        assert_eq!(report_id(c.java_line.as_ref().unwrap()), "reRoll");
        assert!(c.rust_line.is_none());
    }

    #[test]
    fn payload_difference_is_reported_with_both_lines() {
        let j = vec![json!({"reportId":"injury","armorModifiers":["Mighty Blow"]})];
        let r = vec![json!({"reportId":"injury","armorModifiers":["Modifier { name: \"Mighty Blow\" }"]})];
        let c = compare_sequences(&j, &r);
        assert!(!c.identical);
        assert_eq!(c.kind, "payload");
        assert_eq!(c.divergence, 0);
    }
}
