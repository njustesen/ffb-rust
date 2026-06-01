use crate::comparator::CompareResult;

const PROGRESS_FILE: &str = "progress.html";

/// Update progress.html after each seed result.
pub fn update(seed: u64, home_roster: &str, away_roster: &str, result: &CompareResult) {
    let html = match std::fs::read_to_string(PROGRESS_FILE) {
        Ok(s) => s,
        Err(_) => generate_initial_html(),
    };

    let updated = apply_seed_result(html, seed, home_roster, away_roster, result);
    if let Err(e) = std::fs::write(PROGRESS_FILE, &updated) {
        log::warn!("Could not write {PROGRESS_FILE}: {e}");
    }
}

fn apply_seed_result(
    html: String,
    seed: u64,
    home_roster: &str,
    away_roster: &str,
    result: &CompareResult,
) -> String {
    let status = if result.matches { "MATCH" } else { "FAIL" };
    let row_marker = format!("<!-- seed-row-{seed} -->");
    let row_html = format!(
        "{marker}<tr class=\"{cls}\"><td>{seed}</td><td>{home} vs {away}</td>\
         <td>{java_n}</td><td>{rust_n}</td><td>{status}</td>\
         <td title=\"{java_hash}\">{java_short}</td>\
         <td title=\"{rust_hash}\">{rust_short}</td></tr>",
        marker = row_marker,
        cls = if result.matches { "match" } else { "fail" },
        home = home_roster,
        away = away_roster,
        java_n = result.java_event_count,
        rust_n = result.rust_event_count,
        status = status,
        java_hash = result.java_hash,
        rust_hash = result.rust_hash,
        java_short = short_hash(&result.java_hash),
        rust_short = short_hash(&result.rust_hash),
    );

    if html.contains(&row_marker) {
        // Replace existing row
        let start = html.find(&row_marker).unwrap();
        let end = html[start..].find("</tr>").map(|i| start + i + 5).unwrap_or(html.len());
        format!("{}{}{}", &html[..start], row_html, &html[end..])
    } else {
        // Append before the closing </tbody> of the parity table
        let anchor = "</tbody>";
        if let Some(pos) = html.rfind(anchor) {
            format!("{}{}\n{}", &html[..pos], row_html, &html[pos..])
        } else {
            // Fallback: append at end
            format!("{}\n<!-- seed {seed} {status} -->", html)
        }
    }
}

fn short_hash(hash: &str) -> String {
    if hash.len() >= 8 { hash[..8].to_string() } else { hash.to_string() }
}

fn generate_initial_html() -> String {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>FFB Parity Dashboard</title>
<style>
  body { font-family: monospace; background: #1e1e1e; color: #d4d4d4; padding: 1em; }
  h1 { color: #569cd6; }
  table { border-collapse: collapse; width: 100%; margin-top: 1em; }
  th, td { border: 1px solid #444; padding: 4px 8px; text-align: left; }
  th { background: #2d2d2d; color: #9cdcfe; }
  tr.match td { background: #1a3a1a; }
  tr.fail td { background: #3a1a1a; }
  .banner { background: #2d2d2d; border-left: 4px solid #569cd6; padding: 0.5em 1em; margin-bottom: 1em; }
</style>
</head>
<body>
<h1>FFB Parity Dashboard</h1>
<div class="banner" id="focus">Parity testing in progress...</div>
<table>
<thead>
<tr>
  <th>Seed</th><th>Matchup</th><th>Java Events</th><th>Rust Events</th>
  <th>Status</th><th>Java Hash</th><th>Rust Hash</th>
</tr>
</thead>
<tbody>
</tbody>
</table>
</body>
</html>
"#
    .to_string()
}
