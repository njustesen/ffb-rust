use ffb_model::model::team_list::TeamList;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandTeamList`.
/// Sends the lobby team list to the client.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandTeamList {
    /// Java: `fTeamList` — the list of available teams.
    pub team_list: TeamList,
}

impl ServerCommandTeamList {
    pub fn new(team_list: TeamList) -> Self { Self { team_list } }
    pub fn get_team_list(&self) -> &TeamList { &self.team_list }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let cmd = ServerCommandTeamList::new(TeamList::default());
        let _ = cmd.get_team_list();
    }

    #[test]
    fn default_works() {
        let _ = ServerCommandTeamList::default();
    }
}
