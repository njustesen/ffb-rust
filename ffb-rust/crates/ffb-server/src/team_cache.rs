/// 1:1 translation of com.fumbbl.ffb.server.TeamCache.
///
/// Same standalone-mode disk-XML pattern as `roster_cache.rs` (see that module's doc
/// comment for the FUMBBL-mode-JSON-loader-vs-standalone-mode-disk-XML distinction).
///
/// Java keys `teamFiles` by `TeamSkeleton` but never overrides `equals`/`hashCode` on
/// `TeamSkeleton`, so the `Map<TeamSkeleton, File>` behaves like a list of
/// (skeleton, file) pairs under Java's default identity-based `HashMap` semantics —
/// `getTeamById`/`getTeamsForCoach` are linear scans filtering on `getId()`/`getCoach()`,
/// never real hash lookups. This is modeled faithfully here as `Vec<(TeamSkeleton, PathBuf)>`
/// rather than forcing an artificial `Hash`/`Eq` impl onto `TeamSkeleton` that Java itself
/// does not have.
///
/// Phase ZY.2 added `ffb_model::xml::XmlHandler` (a 1:1 port of the SAX-driven
/// `com.fumbbl.ffb.xml.XmlHandler`), so `mapToTeam`'s XML-to-`Team` inflation is now real:
/// lookups return a parsed `Team` instead of raw XML text.
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use ffb_model::model::team::Team;
use ffb_model::model::team_skeleton::TeamSkeleton;
use ffb_model::util::file_iterator::FileIterator;
use ffb_model::xml::{IXmlReadable, XmlHandler};

/// Java: `TeamCache`.
pub struct TeamCache {
    /// Java: `teamFiles` (`Map<TeamSkeleton, File>`) — see module doc comment for why
    /// this is a `Vec` of pairs rather than a `HashMap`.
    team_files: Vec<(TeamSkeleton, PathBuf)>,
}

impl TeamCache {
    pub fn new() -> Self {
        Self { team_files: Vec::new() }
    }

    /// Java: `getTeamById(String teamId, Game game)`.
    ///
    /// Java throws via `.get()` on an empty `Optional` (`NoSuchElementException`) when no
    /// match is found; that is modeled here as `None`.
    pub fn get_team_by_id(&self, team_id: &str) -> Option<io::Result<Team>> {
        self.team_files
            .iter()
            .find(|(skeleton, _)| skeleton.get_id() == team_id)
            .map(|(_, file)| map_to_team(file))
    }

    /// Java: `getSkeleton(String teamId)`.
    pub fn get_skeleton(&self, team_id: &str) -> Option<&TeamSkeleton> {
        self.team_files
            .iter()
            .find(|(skeleton, _)| skeleton.get_id() == team_id)
            .map(|(skeleton, _)| skeleton)
    }

    /// Java: `getTeamsForCoach(String coach, Game game)` — filters by coach, maps to
    /// `Team`s, sorts by `Team.comparatorByName()`.
    pub fn get_teams_for_coach(&self, coach: &str) -> io::Result<Vec<Team>> {
        let mut matches: Vec<Team> = self
            .team_files
            .iter()
            .filter(|(skeleton, _)| skeleton.get_coach() == coach)
            .map(|(_, file)| map_to_team(file))
            .collect::<io::Result<Vec<_>>>()?;
        matches.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(matches)
    }

    /// Java: `init(File pTeamDirectory, IFactorySource source)`.
    ///
    /// The `IFactorySource` parameter exists in Java only to construct
    /// `new TeamSkeleton(source)` (which subclasses `Team(IFactorySource)`); this crate's
    /// `TeamSkeleton` has no such constructor requirement, so there is no parameter to
    /// thread through here.
    pub fn init(&mut self, team_directory: &Path) -> io::Result<()> {
        let mut file_iterator = FileIterator::with_options(team_directory, false, |p| {
            p.extension().and_then(|e| e.to_str()) == Some("xml")
        });
        while let Some(file) = file_iterator.next() {
            let content = fs::read_to_string(&file)?;
            let skeleton = parse_team_skeleton(&content);
            self.team_files.push((skeleton, file));
        }
        Ok(())
    }
}

impl Default for TeamCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Java: `XmlHandler.parse(null, xmlSource, new TeamSkeleton(source))`.
fn parse_team_skeleton(xml: &str) -> TeamSkeleton {
    let parsed = XmlHandler::parse(None, xml, Box::new(TeamSkeleton::default()));
    let mut skeleton = match parsed.into_any().downcast::<TeamSkeleton>() {
        Ok(skeleton) => *skeleton,
        Err(_) => TeamSkeleton::default(),
    };
    skeleton.set_xml_content(xml.to_string());
    skeleton
}

/// Java: `mapToTeam(File)` — `XmlHandler.parse(game, xmlSource, new Team(source))`.
fn map_to_team(file: &Path) -> io::Result<Team> {
    let content = fs::read_to_string(file)?;
    let empty = Team {
        id: String::new(), name: String::new(), race: String::new(),
        roster_id: String::new(), coach: String::new(),
        rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
        prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
        cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0,
        team_value: 0, treasury: 0, special_rules: vec![], players: vec![],
        vampire_lord: false, necromancer: false,
    };
    let parsed = XmlHandler::parse(None, &content, Box::new(empty));
    parsed.into_any().downcast::<Team>().map(|t| *t).map_err(|_| {
        io::Error::new(io::ErrorKind::InvalidData, "Team XML did not parse into a Team")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ffb_team_cache_test_{}", name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn team_xml(id: &str, name: &str, coach: &str, team_value: i32) -> String {
        format!(
            r#"<team id="{}"><coach>{}</coach><name>{}</name><currentTeamValue>{}</currentTeamValue></team>"#,
            id, coach, name, team_value
        )
    }

    #[test]
    fn parse_team_skeleton_extracts_fields() {
        let xml = team_xml("284314", "Reavers", "Kalimar", 1_100_000);
        let skeleton = parse_team_skeleton(&xml);
        assert_eq!(skeleton.get_id(), "284314");
        assert_eq!(skeleton.get_coach(), "Kalimar");
        // TeamSkeleton reads its own "teamValue" tag, not Team's "currentTeamValue" —
        // per TeamSkeleton.java's _XML_TAG_TEAM_VALUE constant (see model/team_skeleton.rs).
        assert_eq!(skeleton.get_xml_content(), xml);
    }

    #[test]
    fn init_populates_team_files() {
        let dir = scratch_dir("init");
        fs::write(dir.join("team_a.xml"), team_xml("1", "Reavers", "Kalimar", 1_000_000)).unwrap();
        fs::write(dir.join("team_b.xml"), team_xml("2", "Ziggurat", "BattleLore", 1_200_000)).unwrap();
        fs::write(dir.join("ignored.txt"), "not xml").unwrap();

        let mut cache = TeamCache::new();
        cache.init(&dir).unwrap();

        assert_eq!(cache.team_files.len(), 2);
        assert!(cache.get_skeleton("1").is_some());
        assert!(cache.get_skeleton("2").is_some());
        assert!(cache.get_skeleton("999").is_none());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn get_team_by_id_returns_parsed_team() {
        let dir = scratch_dir("get_by_id");
        fs::write(dir.join("team_a.xml"), team_xml("1", "Reavers", "Kalimar", 1_000_000)).unwrap();

        let mut cache = TeamCache::new();
        cache.init(&dir).unwrap();

        let team = cache.get_team_by_id("1").expect("team should be found").unwrap();
        assert_eq!(team.name, "Reavers");
        assert_eq!(team.coach, "Kalimar");
        assert_eq!(team.team_value, 1_000_000);
        assert!(cache.get_team_by_id("missing").is_none());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn get_teams_for_coach_filters_and_sorts_by_name() {
        let dir = scratch_dir("for_coach");
        fs::write(dir.join("team_a.xml"), team_xml("1", "Zealots", "Kalimar", 1_000_000)).unwrap();
        fs::write(dir.join("team_b.xml"), team_xml("2", "Amazons", "Kalimar", 1_100_000)).unwrap();
        fs::write(dir.join("team_c.xml"), team_xml("3", "Ziggurat", "BattleLore", 1_200_000)).unwrap();

        let mut cache = TeamCache::new();
        cache.init(&dir).unwrap();

        let teams = cache.get_teams_for_coach("Kalimar").unwrap();
        let names: Vec<&str> = teams.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, vec!["Amazons", "Zealots"]);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn get_team_by_id_parses_players() {
        let dir = scratch_dir("players");
        fs::write(
            dir.join("team_a.xml"),
            r#"<team id="1"><name>Reavers</name><player nr="1" id="p1"><name>Joe</name><positionId>lineman</positionId></player></team>"#,
        ).unwrap();

        let mut cache = TeamCache::new();
        cache.init(&dir).unwrap();

        let team = cache.get_team_by_id("1").expect("team should be found").unwrap();
        assert_eq!(team.players.len(), 1);
        assert_eq!(team.players[0].name, "Joe");
        assert_eq!(team.players[0].position_id, "lineman");

        let _ = fs::remove_dir_all(&dir);
    }
}
