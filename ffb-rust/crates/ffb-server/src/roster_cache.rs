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
/// Phase ZY.2 added `ffb_model::xml::XmlHandler` (a 1:1 port of the SAX-driven
/// `com.fumbbl.ffb.xml.XmlHandler`), so this cache now returns real `Roster`/
/// `RosterSkeleton` objects instead of raw XML text.
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use ffb_model::model::roster::Roster;
use ffb_model::model::roster_skeleton::RosterSkeleton;
use ffb_model::util::file_iterator::FileIterator;
use ffb_model::xml::{IXmlReadable, XmlHandler};

/// Java: `RosterCache`.
pub struct RosterCache {
    /// Java: `rosterFileByRosterId`
    roster_file_by_roster_id: HashMap<String, PathBuf>,
    /// Java: `rosterFileByTeamId`
    roster_file_by_team_id: HashMap<String, PathBuf>,
}

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
    /// `team_id`/`roster_id` stand in for `team.getId()`/`team.getRosterId()`.
    pub fn get_roster_for_team(&self, team_id: &str, roster_id: &str) -> io::Result<Roster> {
        // In newer versions of the XML format, the `<rosterId>` is not used (but is still
        // present). So we first check for the presence of a roster matching the team id,
        // and only if no roster is found, do we fall back to looking up the roster using
        // the original rosterId.
        let roster_file = self
            .roster_file_by_team_id
            .get(team_id)
            .or_else(|| self.roster_file_by_roster_id.get(roster_id));

        match roster_file {
            Some(file) => {
                let content = fs::read_to_string(file)?;
                let parsed = XmlHandler::parse(None, &content, Box::new(Roster {
                    id: String::new(),
                    name: String::new(),
                    race: String::new(),
                    reroll_cost: 0,
                    max_rerolls: 0,
                    positions: vec![],
                    special_rules: vec![],
                    necromancer: false,
                    keywords: vec![],
                }));
                parsed.into_any().downcast::<Roster>().map(|r| *r).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "Roster XML did not parse into a Roster")
                })
            }
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

/// Java: `XmlHandler.parse(null, xmlSource, new RosterSkeleton())`.
fn parse_roster_skeleton(xml: &str) -> RosterSkeleton {
    let parsed = XmlHandler::parse(None, xml, Box::new(RosterSkeleton::default()));
    match parsed.into_any().downcast::<RosterSkeleton>() {
        Ok(skeleton) => *skeleton,
        Err(_) => RosterSkeleton::default(),
    }
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
    fn parse_roster_skeleton_extracts_id_and_team() {
        let xml = r#"<roster id="human" team="284314"><name>Human</name></roster>"#;
        let skeleton = parse_roster_skeleton(xml);
        assert_eq!(skeleton.get_id(), "human");
        assert_eq!(skeleton.get_team_id(), "284314");
    }

    #[test]
    fn parse_roster_skeleton_missing_team_is_empty() {
        let xml = r#"<roster id="human"></roster>"#;
        let skeleton = parse_roster_skeleton(xml);
        assert_eq!(skeleton.get_team_id(), "");
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
        fs::write(
            dir.join("roster_human.xml"),
            r#"<roster id="human" team="284314"><name>Human</name><reRollCost>50000</reRollCost></roster>"#,
        ).unwrap();

        let mut cache = RosterCache::new();
        cache.init(&dir).unwrap();

        let roster = cache.get_roster_for_team("284314", "human").unwrap();
        assert_eq!(roster.name, "Human");
        assert_eq!(roster.reroll_cost, 50_000);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn get_roster_for_team_falls_back_to_roster_id() {
        let dir = scratch_dir("lookup_fallback");
        fs::write(dir.join("roster_orc.xml"), r#"<roster id="orc"><name>Orc</name></roster>"#).unwrap();

        let mut cache = RosterCache::new();
        cache.init(&dir).unwrap();

        // No file is indexed by this team id, so it must fall back to roster id.
        let roster = cache.get_roster_for_team("nonexistent-team", "orc").unwrap();
        assert_eq!(roster.name, "Orc");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn get_roster_for_team_missing_both_errors() {
        let cache = RosterCache::new();
        let result = cache.get_roster_for_team("no-team", "no-roster");
        assert!(result.is_err());
    }

    #[test]
    fn get_roster_for_team_parses_positions() {
        let dir = scratch_dir("positions");
        fs::write(
            dir.join("roster_orc.xml"),
            r#"<roster id="orc">
                <name>Orc</name>
                <position id="orc.lineman">
                    <name>Lineman</name>
                    <quantity>16</quantity>
                    <movement>5</movement>
                    <strength>3</strength>
                    <agility>2</agility>
                    <passing>3</passing>
                    <armour>9</armour>
                </position>
            </roster>"#,
        ).unwrap();

        let mut cache = RosterCache::new();
        cache.init(&dir).unwrap();

        let roster = cache.get_roster_for_team("no-team", "orc").unwrap();
        assert_eq!(roster.positions.len(), 1);
        let lineman = roster.position("orc.lineman").unwrap();
        assert_eq!(lineman.movement, 5);
        assert_eq!(lineman.armour, 9);

        let _ = fs::remove_dir_all(&dir);
    }
}
