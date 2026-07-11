/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerBoxLive.
/// Live variant of TalkHandlerBox — uses DecoratingCommandAdapter, SPEC client, EDIT_STATE privilege.
use std::collections::HashSet;
use ffb_model::model::game::Game;
use ffb_model::model::team::Team;
use ffb_model::util::rng::GameRng;
use crate::handler::talk::decorating_command_adapter::DecoratingCommandAdapter;
use crate::handler::talk::talk_handler::TalkHandler;
use crate::handler::talk::talk_handler_box::TalkHandlerBox;
use crate::handler::talk::talk_requirements::{Client, Environment, Privilege};

pub struct TalkHandlerBoxLive {
    base: TalkHandlerBox,
}

impl TalkHandlerBoxLive {
    /// Java: `TalkHandlerBoxLive()`.
    pub fn new() -> Self {
        let adapter = DecoratingCommandAdapter::new();
        let mut privileges = HashSet::new();
        privileges.insert(Privilege::EditState);
        Self {
            base: TalkHandlerBox::new(&adapter, Client::Spec, Environment::None, privileges),
        }
    }

    pub fn base(&self) -> &TalkHandler { self.base.base() }

    /// Java: `handle` — delegates to TalkHandlerBox with live game settings.
    pub fn handle(&self, game: &mut Game, rng: &mut GameRng, commands: &[String], team: &Team) -> Vec<String> {
        self.base.handle(game, rng, commands, team)
    }
}

impl Default for TalkHandlerBoxLive {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerType, PlayerGender, Rules};
    use ffb_model::types::FieldCoordinate;
    use std::collections::HashSet as Set;

    #[test]
    fn construct() { let _ = TalkHandlerBoxLive::new(); }

    #[test]
    fn handle_delegates_to_base_logic() {
        let h = TalkHandlerBoxLive::new();
        let player = ffb_model::model::player::Player {
            id: "p1".into(), name: "Joe".into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Set::new(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None, is_big_guy: false,
            ..Default::default()
        };
        let team = Team {
            id: "t".into(), name: "Team".into(), race: "Human".into(), roster_id: "human".into(),
            coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![player], vampire_lord: false, necromancer: false,
        };
        let mut game = Game::new(team.clone(), team.clone(), Rules::Bb2025);
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        let mut rng = GameRng::new(1);
        let commands = vec!["/box_home".to_string(), "ko".to_string(), "1".to_string()];
        let info = h.handle(&mut game, &mut rng, &commands, &team);
        assert_eq!(info.len(), 1);
    }
}
