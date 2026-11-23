/// 1:1 translation of com.fumbbl.ffb.server.TeamSetupCache (`init` + `getTeamSetups`).
///
/// Java standalone stores team setups in the DB table that `DbInitializer.initTableTeamSetups`
/// seeds from this very cache (`setups/*.xml`); `UtilServerSetup.loadTeamSetup` then queries the
/// table. The Rust server runs without a DB (BACKLOG §P DEVIATION, same as the DB-less join), so
/// it answers those queries from the cache directly -- the same rows.
use std::fs;
use std::io;
use std::path::Path;

use ffb_model::model::team_setup::TeamSetup;
use ffb_model::util::file_iterator::FileIterator;
use ffb_model::xml::XmlHandler;

#[derive(Default)]
pub struct TeamSetupCache {
    /// Java: `fSetupBySetupKey` (key = team id + name).
    setups: Vec<TeamSetup>,
}

impl TeamSetupCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java: `add(TeamSetup)` -- `put` replaces a setup with the same team id and name.
    pub fn add(&mut self, setup: TeamSetup) {
        self.setups.retain(|s| !(s.team_id == setup.team_id && s.name == setup.name));
        self.setups.push(setup);
    }

    /// Java: `init(File pSetupDirectory)`.
    pub fn init(&mut self, setup_directory: &Path) -> io::Result<()> {
        let mut files = FileIterator::with_options(setup_directory, false, |p| {
            p.extension().and_then(|e| e.to_str()) == Some("xml")
        });
        while let Some(file) = files.next() {
            let content = fs::read_to_string(&file)?;
            let parsed = XmlHandler::parse(None, &content, Box::new(TeamSetup::new()));
            if let Ok(setup) = parsed.into_any().downcast::<TeamSetup>() {
                self.add(*setup);
            }
        }
        Ok(())
    }

    /// `DbTeamSetupsForTeamQuery.execute(team)`: the setup names of a team.
    pub fn names_for_team(&self, team_id: &str) -> Vec<String> {
        let mut names: Vec<String> = self.setups.iter().filter(|s| s.team_id == team_id).map(|s| s.name.clone()).collect();
        names.sort();
        names
    }

    /// `DbTeamSetupsQuery.execute(teamId, setupName)`.
    pub fn get(&self, team_id: &str, name: &str) -> Option<&TeamSetup> {
        self.setups.iter().find(|s| s.team_id == team_id && s.name == name)
    }
}
