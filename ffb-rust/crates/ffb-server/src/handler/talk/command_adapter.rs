/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.CommandAdapter.
use std::collections::HashSet;
use ffb_model::model::game::Game;
use ffb_model::model::team::Team;
use crate::net::session_manager::SessionManager;
use crate::model::received_command::SessionId;

pub trait CommandAdapter {
    /// Java: decorateCommands(Set<String>) — transforms/expands registered command strings.
    fn decorate_commands(&self, input: HashSet<String>) -> HashSet<String>;

    /// Java: determineTeam(Game, SessionManager, Session, String[]) — resolves which team the command applies to.
    fn determine_team<'g>(
        &self,
        game: &'g Game,
        session_manager: &SessionManager,
        session: SessionId,
        commands: &[String],
    ) -> Result<&'g Team, String>;
}

#[cfg(test)]
mod tests {
    // trait — no construct test needed; verified via implementors
    #[test]
    fn trait_exists() {}
}
