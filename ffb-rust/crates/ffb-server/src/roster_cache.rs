/// 1:1 translation of com.fumbbl.ffb.server.RosterCache.
///
/// Roster Cache used when the server is in STANDALONE mode.
///
/// For the cache to work, two files have to exist:
///  1. A team xml in `/ffb-server/teams`.
///  2. A roster xml in `/ffb-server/roster`
///
/// This is the *standalone-mode, disk-XML* roster pipeline (Java: `getRosterForTeam`
/// reads `<roster>` XML files off disk via `XmlHandler`/SAX). It is distinct from the
/// already-existing JSON `data/rosters/*.json` loader in `ffb-model/src/data/loader.rs`,
/// which is the FUMBBL-mode / build-time roster pipeline used everywhere else in this
/// crate. Both are kept separate deliberately, matching the Java source having two
/// unrelated roster mechanisms.
///
/// This crate has no SAX-based `XmlHandler`/general XML parser (that infrastructure was
/// never ported — see `CLAUDE.md`'s translation ground rules on not inventing new
/// infrastructure). Since the only fields `RosterSkeleton` needs are the `id`/`team`
/// attributes of the root `<roster>` element, and the full `Roster` this cache returns
/// is out of scope without a real XML parser, this file:
///   - implements `init` faithfully (walk the directory, extract the root attributes,
///     populate both maps, matching Java's error-handling/precedence exactly), using a
///     narrow attribute-only extractor (`extract_root_attr`) as a documented stand-in
///     for `XmlHandler.parse(null, xmlSource, new RosterSkeleton())`.
///   - implements `get_roster_for_team`'s *lookup* logic (team-id first, falling back to
///     roster-id, then the "not found" error) faithfully, but the final
///     `XmlHandler.parse(game, xmlSource, new Roster())` step — inflating the file
///     contents into a full `Roster` model — is left as a documented gap: there is no
///     Rust XML-to-`Roster` deserializer in this crate, so the resolved file's raw
///     contents are returned instead of a parsed `Roster`, mirroring the same real gap
///     already noted against the other 11 blocked handlers (`RosterCache`/`TeamCache`
///     in TRANSLATION_TRACKER.md's Phase ZW closeout note).
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use ffb_model::model::roster_skeleton::RosterSkeleton;
use ffb_model::util::file_iterator::FileIterator;

/// Java: `RosterCache`.
pub struct RosterCache {
    /// Java: `rosterFileByRosterId`
    roster_file_by_roster_id: HashMap<String, PathBuf>,
    /// Java: `rosterFileByTeamId`
    roster_file_by_team_id: HashMap<String, PathBuf>,
}

/// Result of resolving a roster file for a team: the raw XML text of the resolved
/// roster file. Java would go on to parse this into a full `Roster` via
/// `XmlHandler.parse(game, xmlSource, new Roster())`; no such parser exists in this
/// crate (see module doc comment), so callers get the raw XML instead.
pub type RosterXml = String;

impl RosterCache {
    /// Java: `public RosterCache()`
    pub fn new() -> Self {
        Self {
            roster_file_by_roster_id: HashMap::new(),
            roster_file_by_team_id: HashMap::new(),
        }
    }

    /// Java: `getRosterForTeam(Team team, Game game)`.
    ///
    /// `team_id`/`roster_id` stand in for `team.getId()`/`team.getRosterId()`; `Game` is
    /// not needed since we don't parse into a full `Roster` (see module doc comment).
    pub fn get_roster_for_team(&self, team_id: &str, roster_id: &str) -> io::Result<RosterXml> {
        // In newer versions of the XML format, the `<rosterId>` is not used (but is still
        // present). So we first check for the presence of a roster matching the team id,
        // and only if no roster is found, do we fall back to looking up the roster using
        // the original rosterId.
        let roster_file = self
            .roster_file_by_team_id
            .get(team_id)
            .or_else(|| self.roster_file_by_roster_id.get(roster_id));

        match roster_file {
            Some(file) => fs::read_to_string(file),
            None => Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "No roster found for neither rosterId ({}) nor teamId ({})",
                    roster_id, team_id
                ),
            )),
        }
    }

    /// Java: `init(File pRosterDirectory)`.
    pub fn init(&mut self, roster_directory: &Path) -> io::Result<()> {
        let mut file_iterator = FileIterator::with_options(roster_directory, false, |p| {
            p.extension().and_then(|e| e.to_str()) == Some("xml")
        });
        while let Some(file) = file_iterator.next() {
            let content = fs::read_to_string(&file)?;
            let roster = parse_roster_skeleton(&content);

            if !roster.get_team_id().is_empty() {
                self.roster_file_by_team_id
                    .insert(roster.get_team_id().to_string(), file.clone());
            } else if !roster.get_id().is_empty() {
                self.roster_file_by_roster_id
                    .insert(roster.get_id().to_string(), file.clone());
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "Roster are missing either an 'id' or 'team' attribute: {}",
                        file.display()
                    ),
                ));
            }
        }
        Ok(())
    }
}

impl Default for RosterCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Narrow stand-in for `XmlHandler.parse(null, xmlSource, new RosterSkeleton())`: extracts
/// the `id`/`team` attributes of the root `<roster ...>` element. Not a general XML
/// parser — see module doc comment.
fn parse_roster_skeleton(xml: &str) -> RosterSkeleton {
    let mut skeleton = RosterSkeleton::default();
    if let Some(id) = extract_root_attr(xml, "roster", "id") {
        skeleton.set_id(id);
    }
    if let Some(team_id) = extract_root_attr(xml, "roster", "team") {
        skeleton.set_team_id(team_id);
    }
    skeleton
}

/// Extracts `attr="value"` from the first `<tag ...>` occurrence in `xml`.
fn extract_root_attr(xml: &str, tag: &str, attr: &str) -> Option<String> {
    let open = format!("<{}", tag);
    let start = xml.find(&open)?;
    let close = xml[start..].find('>')? + start;
    let tag_text = &xml[start..close];

    let needle = format!("{}=", attr);
    let mut search_from = 0usize;
    while let Some(rel) = tag_text[search_from..].find(&needle) {
        let attr_start = search_from + rel;
        // Ensure we matched a whole attribute name (preceded by whitespace), not a suffix
        // of a longer attribute name (e.g. "team" inside "teamValue").
        let preceded_by_boundary = tag_text[..attr_start]
            .chars()
            .last()
            .map(|c| c.is_whitespace())
            .unwrap_or(true);
        if preceded_by_boundary {
            let rest = &tag_text[attr_start + needle.len()..];
            let quote = rest.chars().next()?;
            if quote == '"' || quote == '\'' {
                let value_start = 1;
                if let Some(end_rel) = rest[value_start..].find(quote) {
                    return Some(rest[value_start..value_start + end_rel].trim().to_string());
                }
            }
        }
        search_from = attr_start + needle.len();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ffb_roster_cache_test_{}", name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn extract_root_attr_finds_id_and_team() {
        let xml = r#"<roster id="human" team="284314"><name>Human</name></roster>"#;
        assert_eq!(extract_root_attr(xml, "roster", "id").as_deref(), Some("human"));
        assert_eq!(extract_root_attr(xml, "roster", "team").as_deref(), Some("284314"));
    }

    #[test]
    fn extract_root_attr_missing_returns_none() {
        let xml = r#"<roster id="human"></roster>"#;
        assert_eq!(extract_root_attr(xml, "roster", "team"), None);
    }

    #[test]
    fn init_populates_maps_from_fixture_files() {
        let dir = scratch_dir("init");
        fs::write(dir.join("roster_human.xml"), r#"<roster id="human" team="284314"></roster>"#).unwrap();
        fs::write(dir.join("roster_orc.xml"), r#"<roster id="orc"></roster>"#).unwrap();
        fs::write(dir.join("ignored.txt"), "not xml").unwrap();

        let mut cache = RosterCache::new();
        cache.init(&dir).unwrap();

        assert!(cache.roster_file_by_team_id.contains_key("284314"));
        assert!(cache.roster_file_by_roster_id.contains_key("orc"));
        // The human roster has a team id, so it is *not* also indexed by roster id.
        assert!(!cache.roster_file_by_roster_id.contains_key("human"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn get_roster_for_team_prefers_team_id_over_roster_id() {
        let dir = scratch_dir("lookup_team");
        fs::write(dir.join("roster_human.xml"), r#"<roster id="human" team="284314">HUMAN_XML</roster>"#).unwrap();

        let mut cache = RosterCache::new();
        cache.init(&dir).unwrap();

        let xml = cache.get_roster_for_team("284314", "human").unwrap();
        assert!(xml.contains("HUMAN_XML"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn get_roster_for_team_falls_back_to_roster_id() {
        let dir = scratch_dir("lookup_fallback");
        fs::write(dir.join("roster_orc.xml"), r#"<roster id="orc">ORC_XML</roster>"#).unwrap();

        let mut cache = RosterCache::new();
        cache.init(&dir).unwrap();

        // No file is indexed by this team id, so it must fall back to roster id.
        let xml = cache.get_roster_for_team("nonexistent-team", "orc").unwrap();
        assert!(xml.contains("ORC_XML"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn get_roster_for_team_missing_both_errors() {
        let cache = RosterCache::new();
        let result = cache.get_roster_for_team("no-team", "no-roster");
        assert!(result.is_err());
    }
}
