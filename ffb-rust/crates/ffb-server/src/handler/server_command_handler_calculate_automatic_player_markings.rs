/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerCalculateAutomaticPlayerMarkings.
use std::collections::HashMap;
use ffb_engine::marking::auto_marking_config::AutoMarkingConfig;
use ffb_engine::marking::marker_generator::MarkerGenerator;
use ffb_model::enums::NetCommandId;
use ffb_model::model::game::Game;
use crate::model::received_command::SessionId;
use crate::net::session_manager::SessionManager;

pub struct ServerCommandHandlerCalculateAutomaticPlayerMarkings {
    /// Java: `private final MarkerGenerator markerGenerator = new MarkerGenerator();`
    marker_generator: MarkerGenerator,
}

impl ServerCommandHandlerCalculateAutomaticPlayerMarkings {
    pub fn new() -> Self {
        Self {
            marker_generator: MarkerGenerator::new(),
        }
    }

    /// Java: getId() — returns NetCommandId for CALCULATE_AUTOMATIC_PLAYER_MARKINGS.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::InternalCalculateAutomaticPlayerMarkings
    }

    /// Java: handleCommand(ReceivedCommand) — calculates automatic player markings.
    pub fn handle_command(
        &self,
        game: &Game,
        config: &mut AutoMarkingConfig,
        index: i32,
        session_id: SessionId,
        session_manager: &SessionManager,
    ) -> bool {
        // Java: if (config.getMarkings().isEmpty()) config.getMarkings().addAll(AutoMarkingConfig.defaults(...));
        if config.get_markings().is_empty() {
            config.markings = AutoMarkingConfig::defaults();
        }

        let markings = self.handle_game(game, config);

        // Java: getServer().getCommunication().sendMarkings(receivedCommand.getSession(), index, markings);
        send_markings(session_manager, session_id, index, &markings);

        true
    }

    /// Java: `private Map<String, String> handleGame(Game game, AutoMarkingConfig config)`.
    fn handle_game(&self, game: &Game, config: &AutoMarkingConfig) -> HashMap<String, String> {
        let mut markings = HashMap::new();
        for player in game.team_home.players.iter().chain(game.team_away.players.iter()) {
            let marking = self.marker_generator.generate(game, player, config, false);
            markings.insert(player.id.clone(), marking);
        }
        markings
    }
}

/// Java: `Communication.sendMarkings(Session, int, Map<String, String>)`.
fn send_markings(session_manager: &SessionManager, session_id: SessionId, index: i32, markings: &HashMap<String, String>) {
    let json = serde_json::json!({
        "id": "internalCalculateAutomaticPlayerMarkings",
        "index": index,
        "markings": markings,
    })
    .to_string();
    session_manager.send_to(session_id, &json);
}

impl Default for ServerCommandHandlerCalculateAutomaticPlayerMarkings {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_engine::marking::auto_marking_record::Builder;
    use ffb_model::enums::Rules;
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::{SkillId, SkillWithValue};
    use ffb_model::model::team::Team;
    use ffb_model::model::ClientMode;
    use tokio::sync::mpsc;

    fn team(id: &str) -> Team {
        Team {
            id: id.into(),
            name: id.into(),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: "coach".into(),
            rerolls: 0,
            apothecaries: 0,
            bribes: 0,
            master_chefs: 0,
            prayers_to_nuffle: 0,
            bloodweiser_kegs: 0,
            riotous_rookies: 0,
            cheerleaders: 0,
            assistant_coaches: 0,
            fan_factor: 0,
            dedicated_fans: 0,
            team_value: 0,
            treasury: 0,
            special_rules: vec![],
            players: vec![],
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn player(id: &str, extra_skill: Option<SkillId>) -> Player {
        let mut p = Player { id: id.into(), ..Default::default() };
        if let Some(skill) = extra_skill {
            p.extra_skills.push(SkillWithValue::new(skill));
        }
        p
    }

    fn game_with_players() -> Game {
        let mut home = team("home");
        home.players.push(player("p1", Some(SkillId::Block)));
        let mut away = team("away");
        away.players.push(player("p2", None));
        Game::new(home, away, Rules::Bb2025)
    }

    fn config_with_block_marking() -> AutoMarkingConfig {
        let mut config = AutoMarkingConfig::new();
        config.markings.push(
            Builder::new()
                .with_skill(SkillId::Block)
                .with_marking("B")
                .with_gained_only(true)
                .build(),
        );
        config
    }

    #[test]
    fn construct() {
        let _ = ServerCommandHandlerCalculateAutomaticPlayerMarkings::new();
    }

    #[test]
    fn get_id_is_calculate_automatic_player_markings() {
        let h = ServerCommandHandlerCalculateAutomaticPlayerMarkings::new();
        assert_eq!(h.get_id(), NetCommandId::InternalCalculateAutomaticPlayerMarkings);
    }

    #[test]
    fn handle_game_maps_player_ids_to_markings() {
        let h = ServerCommandHandlerCalculateAutomaticPlayerMarkings::new();
        let game = game_with_players();
        let config = config_with_block_marking();
        let markings = h.handle_game(&game, &config);
        assert_eq!(markings.get("p1").map(String::as_str), Some("B"));
        assert_eq!(markings.get("p2").map(String::as_str), Some(""));
    }

    #[test]
    fn handle_command_fills_defaults_and_sends() {
        let h = ServerCommandHandlerCalculateAutomaticPlayerMarkings::new();
        let game = game_with_players();
        let mut config = AutoMarkingConfig::new();
        let mut sm = SessionManager::new();
        let (tx, mut rx) = mpsc::unbounded_channel();
        sm.add_session(1, 100, "Spec".into(), ClientMode::SPECTATOR, false, vec![], tx);

        assert!(h.handle_command(&game, &mut config, 3, 1, &sm));
        assert!(!config.get_markings().is_empty());

        let sent = rx.try_recv().expect("expected a markings message");
        assert!(sent.contains("\"index\":3"));
    }

    #[test]
    fn handle_command_preserves_non_empty_config() {
        let h = ServerCommandHandlerCalculateAutomaticPlayerMarkings::new();
        let game = game_with_players();
        let mut config = config_with_block_marking();
        let before_len = config.get_markings().len();
        let mut sm = SessionManager::new();
        let (tx, _rx) = mpsc::unbounded_channel();
        sm.add_session(1, 100, "Spec".into(), ClientMode::SPECTATOR, false, vec![], tx);

        h.handle_command(&game, &mut config, 0, 1, &sm);
        assert_eq!(config.get_markings().len(), before_len);
    }
}
