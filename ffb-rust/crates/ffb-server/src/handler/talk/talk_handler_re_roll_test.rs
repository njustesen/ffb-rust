/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerReRollTest.
/// Test variant of TalkHandlerReRoll — uses IdentityCommandAdapter, PLAYER client, TEST_GAME env.
use ffb_model::model::game::Game;
use ffb_model::model::team::Team;
use crate::handler::talk::identity_command_adapter::IdentityCommandAdapter;
use crate::handler::talk::talk_handler::TalkHandler;
use crate::handler::talk::talk_handler_re_roll::TalkHandlerReRoll;
use crate::handler::talk::talk_requirements::{Client, Environment};

pub struct TalkHandlerReRollTest {
    base: TalkHandlerReRoll,
}

impl TalkHandlerReRollTest {
    /// Java: `TalkHandlerReRollTest()`.
    pub fn new() -> Self {
        let adapter = IdentityCommandAdapter::new();
        Self {
            base: TalkHandlerReRoll::new(&adapter, Client::Player, Environment::TestGame, Default::default()),
        }
    }

    pub fn base(&self) -> &TalkHandler { self.base.base() }

    /// Java: `handle` — delegates to TalkHandlerReRoll with test game settings.
    pub fn handle(&self, game: &mut Game, commands: &[String], team: &Team) {
        self.base.handle(game, commands, team)
    }
}

impl Default for TalkHandlerReRollTest {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;

    fn empty_team(id: &str) -> Team {
        Team {
            id: id.into(), name: id.into(), race: "Human".into(), roster_id: "human".into(),
            coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false,
        }
    }

    #[test]
    fn construct() { let _ = TalkHandlerReRollTest::new(); }

    #[test]
    fn handle_delegates_to_base_logic() {
        let h = TalkHandlerReRollTest::new();
        let mut game = Game::new(empty_team("home"), empty_team("away"), Rules::Bb2025);
        let team = game.team_away.clone();
        let commands = vec!["/set_rerolls".to_string(), "6".to_string()];
        h.handle(&mut game, &commands, &team);
        assert_eq!(game.turn_data_away.rerolls, 6);
    }
}
