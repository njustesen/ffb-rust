/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerRedeploy.
/// Handles `/redeploy` command — triggers server redeploy (DEV privilege, TEST_SERVER env).
///
/// Java's `TalkHandlerRedeploy(server, branch)` call ultimately resolves the
/// redeploy file path / default branch / process exit code from
/// `server.getProperty(IServerProperty.SERVER_REDEPLOY_*)` (see
/// `redeploy_handler.rs`'s doc comment) — the Rust MVP has no server-properties
/// system yet, so `handle` takes those three values as explicit parameters
/// instead of pulling them from a `FantasyFootballServer`.
use std::collections::HashSet;
use crate::handler::redeploy_handler::RedeployHandler;
use crate::handler::talk::command_adapter::CommandAdapter;
use crate::handler::talk::identity_command_adapter::IdentityCommandAdapter;
use crate::handler::talk::talk_handler::TalkHandler;
use crate::handler::talk::talk_requirements::{Client, Environment, Privilege};

pub struct TalkHandlerRedeploy {
    base: TalkHandler,
    redeploy_handler: RedeployHandler,
}

impl TalkHandlerRedeploy {
    /// Java: `TalkHandlerRedeploy()` — fixed `/redeploy`, threshold 0, PLAYER
    /// client, TEST_SERVER env, DEV privilege, `IdentityCommandAdapter` (the
    /// default used by the `TalkHandler(String, int, Client, Environment,
    /// Privilege...)` super-constructor overload that Java's `super(...)`
    /// call resolves to here).
    pub fn new() -> Self {
        let mut commands = HashSet::new();
        commands.insert("/redeploy".to_string());
        let adapter = IdentityCommandAdapter::new();
        let commands = adapter.decorate_commands(commands);
        let mut privileges = HashSet::new();
        privileges.insert(Privilege::Dev);
        Self {
            base: TalkHandler::new(commands, 0, Client::Player, Environment::TestServer, privileges),
            redeploy_handler: RedeployHandler::new(),
        }
    }

    pub fn base(&self) -> &TalkHandler { &self.base }

    /// Java: `handle(FantasyFootballServer, GameState, String[], Team, Session)` —
    /// extracts the optional branch argument from `commands[1]` and delegates
    /// to `RedeployHandler.redeploy(server, branch)`.
    pub fn handle(&self, commands: &[String], default_branch: &str, redeploy_file: &str, exit_code: i32) {
        let branch = Self::branch_argument(commands);
        self.redeploy_handler.redeploy(branch, default_branch, redeploy_file, exit_code);
    }

    /// Java: `String branch = null; if (commands.length > 1) { branch = commands[1]; }`.
    fn branch_argument(commands: &[String]) -> Option<&str> {
        if commands.len() > 1 {
            Some(commands[1].as_str())
        } else {
            None
        }
    }
}

impl Default for TalkHandlerRedeploy {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() { let _ = TalkHandlerRedeploy::new(); }

    #[test]
    fn branch_argument_returns_none_without_extra_token() {
        let commands = vec!["/redeploy".to_string()];
        assert_eq!(TalkHandlerRedeploy::branch_argument(&commands), None);
    }

    #[test]
    fn branch_argument_returns_second_token_when_present() {
        let commands = vec!["/redeploy".to_string(), "feature-1".to_string()];
        assert_eq!(TalkHandlerRedeploy::branch_argument(&commands), Some("feature-1"));
    }
}
