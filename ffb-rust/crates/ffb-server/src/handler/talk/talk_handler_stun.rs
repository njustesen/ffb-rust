/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerStun.
/// Abstract handler for `/stun` command — places field players into stunned state.
use ffb_model::model::game::Game;
use ffb_model::model::team::Team;
use ffb_model::enums::{PlayerState, PS_STUNNED};
use super::talk_handler_skill::find_player_ids_in_command;
use super::talk_requirements::{Client, Environment, Privilege};

pub struct TalkHandlerStun {
    pub required_client: Client,
    pub required_environment: Environment,
    pub requires_one_privilege_of: Vec<Privilege>,
}

impl TalkHandlerStun {
    /// Java: `super("/stun", 1, ...)`.
    pub const COMMAND: &'static str = "/stun";
    pub const COMMAND_PARTS_THRESHOLD: usize = 1;

    pub fn new(
        required_client: Client,
        required_environment: Environment,
        requires_one_privilege_of: Vec<Privilege>,
    ) -> Self {
        Self { required_client, required_environment, requires_one_privilege_of }
    }

    /// Java: `handle(...)` — every player named in `commands` (or `all`) that isn't in a
    /// dugout box is set to STUNNED and marked active.
    pub fn handle(&self, game: &mut Game, team: &Team, commands: &[String]) -> Vec<String> {
        let mut messages = Vec::new();
        for id in find_player_ids_in_command(team, commands, Self::COMMAND_PARTS_THRESHOLD) {
            let Some(coordinate) = game.field_model.player_coordinate(&id) else { continue };
            if coordinate.is_box_coordinate() {
                continue;
            }
            let name = team.player(&id).map(|p| p.name.clone()).unwrap_or_default();
            game.field_model.set_player_state(&id, PlayerState::new(PS_STUNNED).change_active(true));
            messages.push(format!("Player {name} stunned."));
        }
        messages
    }
}

impl Default for TalkHandlerStun {
    fn default() -> Self {
        Self::new(Client::None, Environment::None, Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, PS_STANDING};
    use ffb_model::model::player::Player;
    use ffb_model::types::{FieldCoordinate, RSV_HOME_X};

    fn game_with_player() -> (Game, Team) {
        let player = Player { id: "p1".into(), name: "Alice".into(), nr: 1, ..Player::default() };
        let team = Team {
            id: "home".into(), name: "Home".into(), race: "Human".into(), roster_id: "human".into(),
            coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![player.clone()], vampire_lord: false, necromancer: false,
        };
        let mut g = Game::new(team.clone(), team.clone(), Rules::Bb2020);
        g.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        g.field_model.set_player_state("p1", PlayerState::new(PS_STANDING));
        (g, team)
    }

    #[test]
    fn construct() {
        let h = TalkHandlerStun::new(Client::Spec, Environment::None, vec![Privilege::EditState]);
        assert_eq!(h.required_client, Client::Spec);
    }

    #[test]
    fn handle_stuns_player_on_pitch() {
        let h = TalkHandlerStun::default();
        let (mut g, team) = game_with_player();
        let commands = vec!["/stun".into(), "1".into()];
        let messages = h.handle(&mut g, &team, &commands);
        assert_eq!(messages, vec!["Player Alice stunned.".to_string()]);
        assert_eq!(g.field_model.player_state("p1").unwrap().base(), PS_STUNNED);
    }

    #[test]
    fn handle_skips_player_in_box() {
        let h = TalkHandlerStun::default();
        let (mut g, team) = game_with_player();
        g.field_model.set_player_coordinate("p1", FieldCoordinate::new(RSV_HOME_X, 3));
        let commands = vec!["/stun".into(), "all".into()];
        assert!(h.handle(&mut g, &team, &commands).is_empty());
    }
}
