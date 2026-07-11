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
/// As with `RosterCache`, there is no ported SAX `XmlHandler` in this crate, so
/// `mapToTeam`'s XML-to-`Team` inflation is left as a documented gap: lookups return the
/// resolved file's raw XML text instead of a parsed `Team`.
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use ffb_model::model::team_skeleton::TeamSkeleton;
use ffb_model::util::file_iterator::FileIterator;

/// Java: `TeamCache`.
pub struct TeamCache {
    /// Java: `teamFiles` (`Map<TeamSkeleton, File>`) — see module doc comment for why
    /// this is a `Vec` of pairs rather than a `HashMap`.
    team_files: Vec<(TeamSkeleton, PathBuf)>,
}

/// Java would return a fully-parsed `Team`; see module doc comment for why this crate
/// returns the raw XML text instead.
pub type TeamXml = String;

impl TeamCache {
    pub fn new() -> Self {
        Self { team_files: Vec::new() }
    }

    /// Java: `getTeamById(String teamId, Game game)`.
    ///
    /// Java throws via `.get()` on an empty `Optional` (`NoSuchElementException`) when no
    /// match is found; that is modeled here as `None`.
    pub fn get_team_by_id(&self, team_id: &str) -> Option<io::Result<TeamXml>> {
        self.team_files
            .iter()
            .find(|(skeleton, _)| skeleton.get_id() == team_id)
            .map(|(_, file)| fs::read_to_string(file))
    }

    /// Java: `getSkeleton(String teamId)`.
    pub fn get_skeleton(&self, team_id: &str) -> Option<&TeamSkeleton> {
        self.team_files
            .iter()
            .find(|(skeleton, _)| skeleton.get_id() == team_id)
            .map(|(skeleton, _)| skeleton)
    }

    /// Java: `getTeamsForCoach(String coach, Game game)` — filters by coach, maps to
    /// `Team`s, sorts by `Team.comparatorByName()`. Since this crate has no XML-to-`Team`
    /// parser, this returns `(name, xml)` pairs sorted by name instead of `Team[]`,
    /// preserving the same filter+sort behavior Java performs.
    pub fn get_teams_for_coach(&self, coach: &str) -> io::Result<Vec<(String, TeamXml)>> {
        let mut matches: Vec<(String, TeamXml)> = self
            .team_files
            .iter()
            .filter(|(skeleton, _)| skeleton.get_coach() == coach)
            .map(|(skeleton, file)| Ok((skeleton.get_name().to_string(), fs::read_to_string(file)?)))
            .collect::<io::Result<Vec<_>>>()?;
        matches.sort_by(|a, b| a.0.cmp(&b.0));
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

/// Narrow stand-in for `XmlHandler.parse(null, xmlSource, new TeamSkeleton(source))`:
/// extracts the `id` attribute of the root `<team ...>` element and the `name`/`coach`/
/// `teamValue` child element text. Not a general XML parser — see module doc comment.
fn parse_team_skeleton(xml: &str) -> TeamSkeleton {
    let mut skeleton = TeamSkeleton::default();
    if let Some(id) = extract_root_attr(xml, "team", "id") {
        skeleton.set_id(id);
    }
    if let Some(name) = extract_element_text(xml, "name") {
        skeleton.set_name(name);
    }
    if let Some(coach) = extract_element_text(xml, "coach") {
        skeleton.set_coach(coach);
    }
    if let Some(team_value) = extract_element_text(xml, "teamValue") {
        if let Ok(value) = team_value.trim().parse::<i32>() {
            skeleton.set_team_value(value);
        }
    }
    skeleton.set_xml_content(xml.to_string());
    skeleton
}

fn extract_root_attr(xml: &str, tag: &str, attr: &str) -> Option<String> {
    let open = format!("<{}", tag);
    let start = xml.find(&open)?;
    let close = xml[start..].find('>')? + start;
    let tag_text = &xml[start..close];

    let needle = format!("{}=", attr);
    let mut search_from = 0usize;
    while let Some(rel) = tag_text[search_from..].find(&needle) {
        let attr_start = search_from + rel;
        let preceded_by_boundary = tag_text[..attr_start]
            .chars()
            .last()
            .map(|c| c.is_whitespace())
            .unwrap_or(true);
        if preceded_by_boundary {
            let rest = &tag_text[attr_start + needle.len()..];
            let quote = rest.chars().next()?;
            if quote == '"' || quote == '\'' {
                if let Some(end_rel) = rest[1..].find(quote) {
                    return Some(rest[1..1 + end_rel].trim().to_string());
                }
            }
        }
        search_from = attr_start + needle.len();
    }
    None
}

fn extract_element_text(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}", tag);
    let start = xml.find(&open)?;
    let open_end = xml[start..].find('>')? + start + 1;
    let close_tag = format!("</{}>", tag);
    let end = xml[open_end..].find(&close_tag)? + open_end;
    Some(xml[open_end..end].trim().to_string())
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
            r#"<team id="{}"><coach>{}</coach><name>{}</name><teamValue>{}</teamValue></team>"#,
            id, coach, name, team_value
        )
    }

    #[test]
    fn parse_team_skeleton_extracts_fields() {
        let xml = team_xml("284314", "Reavers", "Kalimar", 1_100_000);
        let skeleton = parse_team_skeleton(&xml);
        assert_eq!(skeleton.get_id(), "284314");
        assert_eq!(skeleton.get_name(), "Reavers");
        assert_eq!(skeleton.get_coach(), "Kalimar");
        assert_eq!(skeleton.get_team_value(), 1_100_000);
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
    fn get_team_by_id_returns_raw_xml() {
        let dir = scratch_dir("get_by_id");
        fs::write(dir.join("team_a.xml"), team_xml("1", "Reavers", "Kalimar", 1_000_000)).unwrap();

        let mut cache = TeamCache::new();
        cache.init(&dir).unwrap();

        let xml = cache.get_team_by_id("1").expect("team should be found").unwrap();
        assert!(xml.contains("Reavers"));
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
        let names: Vec<&str> = teams.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, vec!["Amazons", "Zealots"]);

        let _ = fs::remove_dir_all(&dir);
    }
}
