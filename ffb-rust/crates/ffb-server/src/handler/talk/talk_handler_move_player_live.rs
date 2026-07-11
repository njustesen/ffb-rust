/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerMovePlayerLive.
/// Live variant of TalkHandlerMovePlayer — uses DecoratingCommandAdapter, SPEC client, EDIT_STATE privilege.
use std::collections::HashSet;
use ffb_model::model::field_model::FieldModel;
use ffb_model::model::team::Team;
use crate::handler::talk::decorating_command_adapter::DecoratingCommandAdapter;
use crate::handler::talk::talk_handler::TalkHandler;
use crate::handler::talk::talk_handler_move_player::TalkHandlerMovePlayer;
use crate::handler::talk::talk_requirements::{Client, Environment, Privilege};
use crate::model::received_command::SessionId;
use crate::net::session_manager::SessionManager;

pub struct TalkHandlerMovePlayerLive {
    base: TalkHandlerMovePlayer,
}

impl TalkHandlerMovePlayerLive {
    /// Java: `TalkHandlerMovePlayerLive()`.
    pub fn new() -> Self {
        let adapter = DecoratingCommandAdapter::new();
        let mut privileges = HashSet::new();
        privileges.insert(Privilege::EditState);
        Self {
            base: TalkHandlerMovePlayer::new(&adapter, Client::Spec, Environment::None, privileges),
        }
    }

    pub fn base(&self) -> &TalkHandler { self.base.base() }

    /// Java: `handle` — delegates to TalkHandlerMovePlayer with live game settings.
    pub fn handle(
        &self,
        field_model: &mut FieldModel,
        commands: &[String],
        team: &Team,
        game_id: i64,
        session: SessionId,
        session_manager: &SessionManager,
    ) -> Vec<String> {
        self.base.handle(field_model, commands, team, game_id, session, session_manager)
    }
}

impl Default for TalkHandlerMovePlayerLive {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::model::ClientMode;
    use ffb_model::types::FieldCoordinate;
    use std::collections::HashSet as Set;

    fn make_player(id: &str, nr: i32) -> ffb_model::model::player::Player {
        ffb_model::model::player::Player {
            id: id.into(), name: format!("Player{nr}"), nr, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Set::new(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None, is_big_guy: false,
            ..Default::default()
        }
    }

    fn make_team(players: Vec<ffb_model::model::player::Player>) -> Team {
        Team {
            id: "t".into(), name: "Team".into(), race: "Human".into(), roster_id: "human".into(),
            coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players, vampire_lord: false, necromancer: false,
        }
    }

    fn session_manager_with(home: SessionId, away: SessionId, game_id: i64) -> SessionManager {
        let mut sm = SessionManager::new();
        let (tx1, _) = tokio::sync::mpsc::unbounded_channel();
        let (tx2, _) = tokio::sync::mpsc::unbounded_channel();
        sm.add_session(home, game_id, "Home".into(), ClientMode::PLAYER, true, vec![], tx1);
        sm.add_session(away, game_id, "Away".into(), ClientMode::PLAYER, false, vec![], tx2);
        sm
    }

    #[test]
    fn construct() { let _ = TalkHandlerMovePlayerLive::new(); }

    #[test]
    fn handle_delegates_to_base_logic() {
        let h = TalkHandlerMovePlayerLive::new();
        let team = make_team(vec![make_player("p1", 1)]);
        let mut fm = FieldModel::default();
        fm.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        let sm = session_manager_with(1, 2, 100);
        let commands = vec!["/move_player_home".to_string(), "1".to_string(), "South".to_string(), "1".to_string()];
        let info = h.handle(&mut fm, &commands, &team, 100, 1, &sm);
        assert_eq!(fm.player_coordinate("p1"), Some(FieldCoordinate::new(5, 6)));
        assert!(!info.is_empty());
    }
}
