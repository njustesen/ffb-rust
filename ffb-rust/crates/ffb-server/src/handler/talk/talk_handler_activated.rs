/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerActivated.
/// Abstract handler for `/set_activated` command — sets active state of field players.
///
/// Java's `handle()` also calls `UtilServerGame.syncGameModel(...)` at the end
/// when privileges are required — that's `TalkHandler.handle()`'s job in the
/// Java template-method pattern and has no direct Rust port target here (no
/// wired sync-to-client infra yet); the caller is responsible for invoking it
/// once available.
use std::collections::HashSet;
use ffb_model::model::field_model::FieldModel;
use ffb_model::model::team::Team;
use ffb_model::enums::PS_STANDING;
use ffb_model::enums::PlayerState;
use crate::handler::talk::command_adapter::CommandAdapter;
use crate::handler::talk::talk_handler::TalkHandler;
use crate::handler::talk::talk_requirements::{Client, Environment, Privilege};

pub struct TalkHandlerActivated {
    base: TalkHandler,
}

impl TalkHandlerActivated {
    /// Java: `TalkHandlerActivated(CommandAdapter, Client, Environment, Privilege...)`.
    pub fn new(
        command_adapter: &dyn CommandAdapter,
        required_client: Client,
        required_env: Environment,
        requires_one_privilege_of: HashSet<Privilege>,
    ) -> Self {
        let mut commands = HashSet::new();
        commands.insert("/set_activated".to_string());
        let commands = command_adapter.decorate_commands(commands);
        Self {
            base: TalkHandler::new(commands, 2, required_client, required_env, requires_one_privilege_of),
        }
    }

    pub fn base(&self) -> &TalkHandler { &self.base }

    /// Java: `handle(FantasyFootballServer, GameState, String[], Team, Session)` —
    /// sets active state of players referenced in the command; returns the
    /// info messages Java would have sent via `communication.sendPlayerTalk`.
    pub fn handle(&self, field_model: &mut FieldModel, commands: &[String], team: &Team) -> Vec<String> {
        let activated: bool = commands[1].parse().unwrap_or(false);
        let mut info = Vec::new();
        let player_ids: Vec<String> = self.base.find_players_in_command(team, commands)
            .into_iter().map(|p| p.id.clone()).collect();
        for player_id in player_ids {
            let player = team.player(&player_id).expect("player must exist in team");
            let coordinate = field_model.player_coordinate(&player_id);
            let in_box = coordinate.map(|c| c.is_box_coordinate()).unwrap_or(false);
            if !in_box {
                let verb = if activated { "activated" } else { "not activated" };
                info.push(format!("Player {} set {verb}.", player.name));
                let state = field_model.player_state(&player_id).unwrap_or(PlayerState::new(PS_STANDING));
                field_model.set_player_state(&player_id, state.change_active(!activated));
            }
        }
        info
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;
    use crate::handler::talk::identity_command_adapter::IdentityCommandAdapter;
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

    fn handler() -> TalkHandlerActivated {
        let adapter = IdentityCommandAdapter::new();
        TalkHandlerActivated::new(&adapter, Client::Player, Environment::None, HashSet::new())
    }

    #[test]
    fn construct() { let _ = handler(); }

    #[test]
    fn handle_activates_player_on_field() {
        let h = handler();
        let team = make_team(vec![make_player("p1", 1)]);
        let mut fm = FieldModel::default();
        fm.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        let commands = vec!["/set_activated".to_string(), "true".to_string(), "1".to_string()];
        let info = h.handle(&mut fm, &commands, &team);
        assert_eq!(info.len(), 1);
        assert!(info[0].contains("set activated"));
        // Java: changeActive(!activated) — the active bit is set to the
        // opposite of the "activated" flag (matches TalkHandlerActivated.java literally).
        assert!(!fm.player_state("p1").unwrap().is_active());
    }

    #[test]
    fn handle_deactivates_player() {
        let h = handler();
        let team = make_team(vec![make_player("p1", 1)]);
        let mut fm = FieldModel::default();
        fm.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        fm.set_player_state("p1", PlayerState::new(PS_STANDING).change_active(true));
        let commands = vec!["/set_activated".to_string(), "false".to_string(), "1".to_string()];
        let info = h.handle(&mut fm, &commands, &team);
        assert!(info[0].contains("not activated"));
        assert!(fm.player_state("p1").unwrap().is_active());
    }

    #[test]
    fn handle_skips_players_in_box() {
        let h = handler();
        let team = make_team(vec![make_player("p1", 1)]);
        let mut fm = FieldModel::default();
        fm.set_player_coordinate("p1", FieldCoordinate::new(ffb_model::types::RSV_HOME_X, 0)); // box column
        let commands = vec!["/set_activated".to_string(), "true".to_string(), "1".to_string()];
        let info = h.handle(&mut fm, &commands, &team);
        assert!(info.is_empty());
    }

    #[test]
    fn handle_all_keyword_targets_every_player() {
        let h = handler();
        let team = make_team(vec![make_player("p1", 1), make_player("p2", 2)]);
        let mut fm = FieldModel::default();
        fm.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        fm.set_player_coordinate("p2", FieldCoordinate::new(6, 6));
        let commands = vec!["/set_activated".to_string(), "true".to_string(), "all".to_string()];
        let info = h.handle(&mut fm, &commands, &team);
        assert_eq!(info.len(), 2);
    }
}
