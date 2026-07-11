/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerUsedActions.
/// Abstract handler for `/action_used` command — marks special team actions as used or unused.
use ffb_model::model::game::Game;
use ffb_model::model::team::Team;
use ffb_model::enums::PlayerAction;
use super::talk_requirements::{Client, Environment, Privilege};

/// Java: `TalkHandlerUsedActions.ACTIONS` — the subset of `PlayerAction` this command
/// can toggle.
const ACTIONS: &[PlayerAction] = &[
    PlayerAction::Foul,
    PlayerAction::Blitz,
    PlayerAction::Pass,
    PlayerAction::HandOver,
    PlayerAction::ThrowBomb,
    PlayerAction::KickTeamMate,
    PlayerAction::ThrowTeamMate,
];

pub struct TalkHandlerUsedActions {
    pub required_client: Client,
    pub required_environment: Environment,
    pub requires_one_privilege_of: Vec<Privilege>,
}

impl TalkHandlerUsedActions {
    /// Java: `super("/action_used", 2, ...)`.
    pub const COMMAND: &'static str = "/action_used";
    pub const COMMAND_PARTS_THRESHOLD: usize = 2;

    pub fn new(
        required_client: Client,
        required_environment: Environment,
        requires_one_privilege_of: Vec<Privilege>,
    ) -> Self {
        Self { required_client, required_environment, requires_one_privilege_of }
    }

    /// Java: `handle(...)` — `commands[1]` is the boolean used-flag, and every trailing
    /// action name (or `all`) toggles the matching flag on the routed team's `TurnData`.
    pub fn handle(&self, game: &mut Game, team: &Team, commands: &[String]) -> Vec<String> {
        let Some(used_token) = commands.get(1) else { return Vec::new() };
        let used = used_token.eq_ignore_ascii_case("true");

        let turn_data = if team.id == game.team_home.id {
            &mut game.turn_data_home
        } else {
            &mut game.turn_data_away
        };

        let mut messages = Vec::new();
        for action in Self::find_actions(commands) {
            match action {
                PlayerAction::Foul => turn_data.foul_used = used,
                PlayerAction::Blitz => turn_data.blitz_used = used,
                PlayerAction::Pass => turn_data.pass_used = used,
                PlayerAction::HandOver => turn_data.hand_over_used = used,
                PlayerAction::ThrowBomb => turn_data.bomb_used = used,
                PlayerAction::KickTeamMate => turn_data.ktm_used = used,
                PlayerAction::ThrowTeamMate => turn_data.ttm_used = used,
                _ => continue,
            }
            let state = if used { "used" } else { "not used" };
            messages.push(format!("Set {} to {state} for {}.", action.name(), team.name));
        }
        messages
    }

    /// Java: `findActions(Team, String[])`.
    fn find_actions(commands: &[String]) -> Vec<PlayerAction> {
        let mut actions = Vec::new();
        if commands.len() > Self::COMMAND_PARTS_THRESHOLD {
            if commands[Self::COMMAND_PARTS_THRESHOLD].eq_ignore_ascii_case("all") {
                actions.extend_from_slice(ACTIONS);
            } else {
                for token in &commands[Self::COMMAND_PARTS_THRESHOLD..] {
                    if let Some(&action) = ACTIONS.iter().find(|a| a.name().eq_ignore_ascii_case(token)) {
                        actions.push(action);
                    }
                }
            }
        }
        actions
    }
}

impl Default for TalkHandlerUsedActions {
    fn default() -> Self {
        Self::new(Client::None, Environment::None, Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;

    fn team(id: &str, name: &str) -> Team {
        Team {
            id: id.into(), name: name.into(), race: "Human".into(), roster_id: "human".into(),
            coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false,
        }
    }

    fn game() -> Game {
        Game::new(team("home", "Home"), team("away", "Away"), Rules::Bb2020)
    }

    #[test]
    fn construct() {
        let h = TalkHandlerUsedActions::new(Client::Spec, Environment::None, vec![Privilege::EditState]);
        assert_eq!(h.required_client, Client::Spec);
    }

    #[test]
    fn handle_marks_single_action_used() {
        let h = TalkHandlerUsedActions::default();
        let mut g = game();
        let home = g.team_home.clone();
        let commands = vec!["/action_used".into(), "true".into(), "foul".into()];
        let messages = h.handle(&mut g, &home, &commands);
        assert!(g.turn_data_home.foul_used);
        assert_eq!(messages, vec!["Set foul to used for Home.".to_string()]);
    }

    #[test]
    fn handle_marks_all_actions_not_used() {
        let h = TalkHandlerUsedActions::default();
        let mut g = game();
        g.turn_data_away.blitz_used = true;
        let away = g.team_away.clone();
        let commands = vec!["/action_used".into(), "false".into(), "all".into()];
        let messages = h.handle(&mut g, &away, &commands);
        assert!(!g.turn_data_away.blitz_used);
        assert_eq!(messages.len(), ACTIONS.len());
    }

    #[test]
    fn handle_unknown_action_is_ignored() {
        let h = TalkHandlerUsedActions::default();
        let mut g = game();
        let home = g.team_home.clone();
        let commands = vec!["/action_used".into(), "true".into(), "not_an_action".into()];
        assert!(h.handle(&mut g, &home, &commands).is_empty());
    }
}
