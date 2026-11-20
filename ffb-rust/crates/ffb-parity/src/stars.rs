//! Star-player tier (BACKLOG §L): field ONE star (a pair counts as one hire) per team on top of
//! the R6-legal squad, identically in both engines.
//!
//! The catalogue is `data/star_players/<edition>.json`, built by `scripts/build_star_catalog.py`
//! from the rulebook pages (bb2020/bb2025) and FFB's original LRB6 rosters (bb2016). Each entry
//! carries the star's roster position twice: as roster JSON (Rust) and as roster XML (Java).
//!
//! Rust: `make_team` appends the star after building the squad (`FFB_HOME_STAR`/`FFB_AWAY_STAR`).
//! Java: [`prepare_java_server_dir`] writes a PRIVATE server dir for the matchup -- every roster
//! XML (the star positions spliced into the squads' rosters) plus the two team XMLs (the star
//! players appended) -- and the runner points `FFB_SERVER_DIR` at it. Java's TeamCache parses
//! every file in `teams/`, so variants are never added to the shared tree.
//!
//! Team value: Java's `HeadlessGameSetup` recomputes TV with `UtilTeamValue.findTeamValue`, which
//! sums every player's position cost, so the star's cost is in Java's TV. Rust adds the same.

use ffb_model::data::loader::position_json_to_roster_position;
use ffb_model::data::roster_json::PositionJson;
use ffb_model::enums::Rules;
use ffb_model::model::player::Player;
use ffb_model::model::team::Team;

#[derive(serde::Deserialize)]
pub struct StarCatalog {
    pub edition: String,
    pub stars: Vec<StarEntry>,
}

#[derive(serde::Deserialize)]
pub struct StarEntry {
    pub id: String,
    pub name: String,
    pub cost: i32,
    pub players: Vec<StarPlayer>,
    pub eligible: Vec<String>,
    #[serde(default)]
    pub dropped_skills: Vec<String>,
}

#[derive(serde::Deserialize)]
pub struct StarPlayer {
    pub position: PositionJson,
    pub position_xml: String,
}

fn rules_of(edition: &str) -> Rules {
    match edition {
        "bb2016" => Rules::Bb2016,
        "bb2020" => Rules::Bb2020,
        _ => Rules::Bb2025,
    }
}

/// `data/star_players/<edition>.json` (env `FFB_STARS_DIR` overrides the directory).
pub fn catalog_path(edition: &str) -> Option<std::path::PathBuf> {
    let file = format!("{edition}.json");
    if let Ok(root) = std::env::var("FFB_STARS_DIR") {
        let p = std::path::Path::new(&root).join(&file);
        return p.exists().then_some(p);
    }
    [
        "data/star_players",
        "../data/star_players",
        "../../data/star_players",
        r"C:\Users\Admin\niels\ffb-rust\ffb-rust\data\star_players",
    ]
    .iter()
    .map(|c| std::path::Path::new(c).join(&file))
    .find(|p| p.exists())
}

pub fn load_catalog(edition: &str) -> Result<StarCatalog, String> {
    let path = catalog_path(edition).ok_or_else(|| format!("no data/star_players/{edition}.json"))?;
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

/// The star requested for `side` ("home"/"away"), if any.
pub fn star_for(side: &str) -> Option<String> {
    let var = if side == "away" { "FFB_AWAY_STAR" } else { "FFB_HOME_STAR" };
    std::env::var(var).ok().filter(|s| !s.is_empty())
}

/// Jersey numbers for the star's players: the next free shirts after the squad.
fn star_numbers(team_nrs: impl Iterator<Item = i32>, count: usize) -> Vec<i32> {
    let max = team_nrs.max().unwrap_or(0);
    (1..=count as i32).map(|k| max + k).collect()
}

/// The squad whose star eligibility a squad has: a §M skill-suite variant (`<base>_sk`,
/// `<base>_st`, drafted by scripts/draft_skill_squads.py / draft_stat_squads.py) is the same race
/// and roster as its base squad, so it may hire exactly the base squad's stars.
pub fn hiring_squad(squad: &str) -> &str {
    squad.strip_suffix("_sk").or_else(|| squad.strip_suffix("_st")).unwrap_or(squad)
}

/// Append `star_id` to `team` (built from squad `squad`). Errors when the star is unknown or not
/// hireable by the squad -- a star schedule must never field an illegal hire.
pub fn apply_star(team: &mut Team, side: &str, edition: &str, squad: &str, star_id: &str) -> Result<(), String> {
    let cat = load_catalog(edition)?;
    let star = cat.stars.iter().find(|s| s.id == star_id)
        .ok_or_else(|| format!("{edition}: unknown star '{star_id}'"))?;
    if !star.eligible.iter().any(|e| e == hiring_squad(squad)) {
        return Err(format!("{edition}: star '{star_id}' is not hireable by '{squad}'"));
    }
    let nrs = star_numbers(team.players.iter().map(|p| p.nr), star.players.len());
    for (sp, nr) in star.players.iter().zip(nrs) {
        let rp = position_json_to_roster_position(&sp.position, &team.roster_id, false, rules_of(edition));
        team.team_value += sp.position.cost;
        team.players.push(Player::from_position(
            format!("{side}_{nr:02}"),
            format!("{} {} {}", side, rp.name, nr),
            nr,
            &rp,
        ));
    }
    team.players.sort_by_key(|p| p.nr);
    Ok(())
}

fn xml_attr<'a>(text: &'a str, open_tag: &str, attr: &str) -> Option<&'a str> {
    let start = text.find(open_tag)?;
    let rest = &text[start..];
    let key = format!("{attr}=\"");
    let a = rest.find(&key)? + key.len();
    let b = rest[a..].find('"')? + a;
    Some(&rest[a..b])
}

fn xml_text<'a>(text: &'a str, tag: &str) -> Option<&'a str> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let a = text.find(&open)? + open.len();
    let b = text[a..].find(&close)? + a;
    Some(&text[a..b])
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// Write the private Java server dir for one star matchup and return its path.
///
/// `base` is the shared ffb-server dir. Every `rosters/*.xml` is copied (RosterCache resolves a
/// team's roster by id across the whole directory); the rosters the two squads use gain the star
/// positions; only the two team XMLs are written to `teams/`, with the star players appended.
pub fn prepare_java_server_dir(
    base: &std::path::Path,
    out: &std::path::Path,
    edition: &str,
    home: (&str, &str, Option<&str>),
    away: (&str, &str, Option<&str>),
) -> Result<std::path::PathBuf, String> {
    let cat = load_catalog(edition)?;
    let rosters_out = out.join("rosters");
    let teams_out = out.join("teams");
    if out.exists() {
        std::fs::remove_dir_all(out).map_err(|e| format!("{}: {e}", out.display()))?;
    }
    std::fs::create_dir_all(&rosters_out).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&teams_out).map_err(|e| e.to_string())?;

    let suffix = match edition { "bb2016" => "16", "bb2020" => "20", _ => "25" };
    // roster id -> positions to splice in (deduplicated by position id).
    let mut splice: std::collections::BTreeMap<String, Vec<(String, String)>> = Default::default();
    for (side, (squad, team_id, star)) in [("home", home), ("away", away)] {
        let src = base.join("teams").join(format!("team_{squad}_parity{suffix}_{side}.xml"));
        let mut text = std::fs::read_to_string(&src).map_err(|e| format!("{}: {e}", src.display()))?;
        if let Some(star_id) = star {
            let entry = cat.stars.iter().find(|s| s.id == star_id)
                .ok_or_else(|| format!("{edition}: unknown star '{star_id}'"))?;
            if !entry.eligible.iter().any(|e| e == hiring_squad(squad)) {
                return Err(format!("{edition}: star '{star_id}' is not hireable by '{squad}'"));
            }
            let roster_id = xml_text(&text, "rosterId").ok_or("team XML without rosterId")?.to_string();
            let nrs: Vec<i32> = text.match_indices("<player nr=\"")
                .filter_map(|(i, m)| {
                    let rest = &text[i + m.len()..];
                    rest[..rest.find('"')?].parse::<i32>().ok()
                })
                .collect();
            let new_nrs = star_numbers(nrs.into_iter(), entry.players.len());
            let mut players_xml = String::new();
            for (sp, nr) in entry.players.iter().zip(new_nrs) {
                players_xml.push_str(&format!(
                    "\t<player nr=\"{nr}\" id=\"{team_id}{nr}\">\n\t\t<name>{} {nr}</name>\n\t\t<gender>male</gender>\n\t\t<positionId>{}</positionId>\n\t\t<skillList/>\n\t\t<playerStatistics currentSpps=\"0\"/>\n\t</player>\n",
                    escape_xml(&sp.position.name), escape_xml(&sp.position.id)));
                let list = splice.entry(roster_id.clone()).or_default();
                if !list.iter().any(|(id, _)| id == &sp.position.id) {
                    list.push((sp.position.id.clone(), sp.position_xml.clone()));
                }
            }
            let at = text.rfind("</team>").ok_or("team XML without </team>")?;
            text.insert_str(at, &players_xml);
        }
        std::fs::write(teams_out.join(src.file_name().unwrap()), text).map_err(|e| e.to_string())?;
    }

    let mut spliced = 0usize;
    for ent in std::fs::read_dir(base.join("rosters")).map_err(|e| e.to_string())? {
        let ent = ent.map_err(|e| e.to_string())?;
        let path = ent.path();
        if path.extension().and_then(|e| e.to_str()) != Some("xml") {
            continue;
        }
        let mut text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        if let Some(id) = xml_attr(&text, "<roster", "id").map(str::to_string) {
            if let Some(list) = splice.get(&id) {
                let at = text.rfind("</roster>").ok_or("roster XML without </roster>")?;
                let block: String = list.iter().map(|(_, x)| x.as_str()).collect();
                text.insert_str(at, &block);
                spliced += 1;
            }
        }
        std::fs::write(rosters_out.join(ent.file_name()), text).map_err(|e| e.to_string())?;
    }
    if spliced != splice.len() {
        return Err(format!("star rosters: spliced {spliced} of {} roster ids {:?}", splice.len(),
            splice.keys().collect::<Vec<_>>()));
    }
    Ok(out.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::SkillId;

    /// Every skill name in every star catalogue must resolve in RUST too: the catalogue builder
    /// already keeps only names Java resolves, so a Rust miss here is a star whose rules exist in
    /// the oracle but would silently be absent from the Rust player.
    #[test]
    fn every_star_skill_resolves_in_rust() {
        let mut missing = Vec::new();
        for ed in ["bb2016", "bb2020", "bb2025"] {
            let cat = load_catalog(ed).expect("catalogue");
            assert_eq!(cat.edition, ed);
            for s in &cat.stars {
                for p in &s.players {
                    for sk in &p.position.skills {
                        if SkillId::from_class_name(sk.name()).is_none() {
                            missing.push(format!("{ed} {} {}", s.id, sk.name()));
                        }
                    }
                }
            }
        }
        assert!(missing.is_empty(), "unresolved star skills: {missing:#?}");
    }

    #[test]
    fn every_star_is_hireable_by_an_existing_squad() {
        for ed in ["bb2016", "bb2020", "bb2025"] {
            for s in &load_catalog(ed).unwrap().stars {
                assert!(!s.eligible.is_empty(), "{ed} {}", s.id);
                for sq in &s.eligible {
                    assert!(crate::runner::make_team_from_file(sq, "home", ed).is_ok(), "{ed} {} -> {sq}", s.id);
                }
            }
        }
    }

    #[test]
    fn apply_star_appends_after_the_squad_and_adds_its_cost() {
        let mut team = crate::runner::make_team_from_file("human", "home", "bb2025").unwrap();
        let before_tv = team.team_value;
        let max_nr = team.players.iter().map(|p| p.nr).max().unwrap();
        apply_star(&mut team, "home", "bb2025", "human", "star.griff_oberwald").unwrap();
        let griff = team.players.last().unwrap();
        assert_eq!(griff.nr, max_nr + 1);
        assert_eq!(griff.id, format!("home_{:02}", max_nr + 1));
        assert_eq!(team.team_value, before_tv + 300_000);
        assert!(apply_star(&mut team, "home", "bb2025", "human", "star.no_such_star").is_err());
        let mut orc = crate::runner::make_team_from_file("undead", "home", "bb2025").unwrap();
        assert!(apply_star(&mut orc, "home", "bb2025", "undead", "star.griff_oberwald").is_err(),
            "Griff does not play for Shambling Undead");
    }

    /// §M 2b(p): a skill-suite variant hires its base squad's stars, and every entry of the
    /// skill-suite hire table (`data/teams_skills/star_hires.json`) is a legal hire.
    #[test]
    fn skill_suite_star_hires_are_legal() {
        assert_eq!(hiring_squad("elf_sk"), "elf");
        assert_eq!(hiring_squad("human_st"), "human");
        assert_eq!(hiring_squad("dark_elf"), "dark_elf");
        let path = catalog_path("bb2025").unwrap().parent().unwrap().parent().unwrap()
            .join("teams_skills").join("star_hires.json");
        let table: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let mut n = 0;
        for (ed, hires) in &table {
            if ed.starts_with('_') { continue; }
            for (squad, star) in hires.as_object().unwrap() {
                let star = star.as_str().unwrap();
                let mut team = crate::runner::make_team_from_file(squad, "home", ed).unwrap();
                apply_star(&mut team, "home", ed, squad, star).unwrap_or_else(|e| panic!("{e}"));
                n += 1;
            }
        }
        assert!(n >= 2, "expected the Jeremiah Kool hires, found {n}");
    }
}
