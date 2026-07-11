/// 1:1 translation of com.fumbbl.ffb.server.handler.ServerCommandHandlerApplyAutomatedPlayerMarkings.
use ffb_engine::marking::auto_marking_config::AutoMarkingConfig;
use ffb_engine::marking::marker_generator::MarkerGenerator;
use ffb_model::enums::NetCommandId;
use ffb_model::marking::player_marker::PlayerMarker;
use ffb_model::model::game::Game;
use ffb_model::model::team::Team;
use ffb_model::model::ClientMode;
use crate::game_cache::GameCache;
use crate::model::received_command::SessionId;
use crate::net::session_manager::SessionManager;

pub struct ServerCommandHandlerApplyAutomatedPlayerMarkings {
    /// Java: `private final MarkerGenerator markerGenerator = new MarkerGenerator();`
    marker_generator: MarkerGenerator,
}

impl ServerCommandHandlerApplyAutomatedPlayerMarkings {
    pub fn new() -> Self {
        Self {
            marker_generator: MarkerGenerator::new(),
        }
    }

    /// Java: getId() — returns NetCommandId for APPLY_AUTOMATED_PLAYER_MARKINGS.
    pub fn get_id(&self) -> NetCommandId {
        NetCommandId::InternalApplyAutomaticPlayerMarkings
    }

    /// Java: handleCommand(ReceivedCommand) — applies automated player markings.
    pub fn handle_command(
        &self,
        config: &mut AutoMarkingConfig,
        game_id: i64,
        session_id: SessionId,
        game_cache: &GameCache,
        session_manager: &SessionManager,
    ) -> bool {
        let game_state = match game_cache.get_game_state_by_id(game_id) {
            Some(gs) => gs,
            None => {
                log::error!("game {}: gamestate not found for ApplyAutomatedPlayerMarkings", game_id);
                return true;
            }
        };

        let game = match game_state.get_game() {
            Some(g) => g,
            None => {
                log::error!("game {}: game not started for ApplyAutomatedPlayerMarkings", game_id);
                return true;
            }
        };

        // Java: if (config.getMarkings().isEmpty()) config.getMarkings().addAll(AutoMarkingConfig.defaults(...));
        if config.get_markings().is_empty() {
            config.markings = AutoMarkingConfig::defaults();
        }

        match session_manager.get_mode_for_session(session_id) {
            Some(ClientMode::PLAYER) => {
                self.mark_for_player(session_manager, game, config, game_id, session_id);
            }
            Some(ClientMode::SPECTATOR) | Some(ClientMode::REPLAY) => {
                self.mark_for_spec_or_replay(game, config, session_id, session_manager);
            }
            _ => {}
        }

        true
    }

    /// Java: `markForSpecOrReplay(GameState, Game, AutoMarkingConfig, Session)`.
    fn mark_for_spec_or_replay(
        &self,
        game: &Game,
        config: &AutoMarkingConfig,
        session_id: SessionId,
        session_manager: &SessionManager,
    ) {
        let markers: Vec<PlayerMarker> = all_players(game)
            .map(|player| {
                let marking = self.marker_generator.generate(game, player, config, false);
                let mut marker = PlayerMarker::with_player_id(player.id.clone());
                marker.set_home_text(marking);
                marker
            })
            .collect();

        // Java: gameState.getServer().getCommunication().sendUpdateLocalPlayerMarkers(session, markers);
        send_update_local_player_markers(session_manager, session_id, &markers);
    }

    /// Java: `markForPlayer(GameState, SessionManager, Game, AutoMarkingConfig, Session)`.
    ///
    /// Java persists each marker into `game.getFieldModel()` and then broadcasts the full model
    /// via `UtilServerGame.syncGameModel`. The Rust `FieldModel` has no marker-storage map yet and
    /// there is no `syncGameModel` broadcast, so the computed markers are sent directly to the
    /// requesting session instead (the same channel used for the spectator/replay case) — the
    /// marking calculation itself is fully ported.
    fn mark_for_player(
        &self,
        session_manager: &SessionManager,
        game: &Game,
        config: &AutoMarkingConfig,
        game_id: i64,
        session_id: SessionId,
    ) {
        let home_coach = session_manager.get_session_of_home_coach(game_id) == Some(session_id);
        let team: &Team = if home_coach { &game.team_home } else { &game.team_away };

        let markers: Vec<PlayerMarker> = all_players(game)
            .map(|player| {
                let marking = self.marker_generator.generate(game, player, config, team.has_player(&player.id));
                let mut marker = PlayerMarker::with_player_id(player.id.clone());
                if home_coach {
                    marker.set_home_text(marking);
                } else {
                    marker.set_away_text(marking);
                }
                marker
            })
            .collect();

        send_update_local_player_markers(session_manager, session_id, &markers);
    }
}

/// Java: `Arrays.stream(game.getPlayers())` — all players of both teams.
fn all_players(game: &Game) -> impl Iterator<Item = &ffb_model::model::player::Player> {
    game.team_home.players.iter().chain(game.team_away.players.iter())
}

/// Java: `Communication.sendUpdateLocalPlayerMarkers(Session, List<PlayerMarker>)`.
///
/// The `ServerCommandUpdateLocalPlayerMarkers` wire struct does not derive `Serialize`, so the
/// JSON payload is built directly from the marker getters.
pub(crate) fn send_update_local_player_markers(
    session_manager: &SessionManager,
    session_id: SessionId,
    markers: &[PlayerMarker],
) {
    let payload: Vec<serde_json::Value> = markers
        .iter()
        .map(|m| {
            serde_json::json!({
                "playerId": m.get_player_id(),
                "homeText": m.get_home_text(),
                "awayText": m.get_away_text(),
            })
        })
        .collect();
    let json = serde_json::json!({
        "id": "serverUpdateLocalPlayerMarkers",
        "markers": payload,
    })
    .to_string();
    session_manager.send_to(session_id, &json);
}

impl Default for ServerCommandHandlerApplyAutomatedPlayerMarkings {
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
        let mut p = Player {
            id: id.into(),
            ..Default::default()
        };
        if let Some(skill) = extra_skill {
            p.extra_skills.push(SkillWithValue::new(skill));
        }
        p
    }

    fn game_with_marked_player() -> Game {
        let mut home = team("home");
        home.players.push(player("p1", Some(SkillId::Block)));
        let away = team("away");
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
        let _ = ServerCommandHandlerApplyAutomatedPlayerMarkings::new();
    }

    #[test]
    fn get_id_is_apply_automatic_player_markings() {
        let h = ServerCommandHandlerApplyAutomatedPlayerMarkings::new();
        assert_eq!(h.get_id(), NetCommandId::InternalApplyAutomaticPlayerMarkings);
    }

    #[test]
    fn handle_command_missing_gamestate_returns_true() {
        let h = ServerCommandHandlerApplyAutomatedPlayerMarkings::new();
        let cache = GameCache::new();
        let sm = SessionManager::new();
        let mut config = AutoMarkingConfig::new();
        assert!(h.handle_command(&mut config, 999, 1, &cache, &sm));
    }

    #[test]
    fn handle_command_spectator_sends_markers() {
        let h = ServerCommandHandlerApplyAutomatedPlayerMarkings::new();
        let mut cache = GameCache::new();
        let game_id = cache.create_game_state();
        cache
            .get_game_state_by_id_mut(game_id)
            .unwrap()
            .start_game(team("home"), team("away"), Rules::Bb2025, 0);
        cache
            .get_game_state_by_id_mut(game_id)
            .unwrap()
            .get_game_mut()
            .unwrap()
            .team_home
            .players
            .push(player("p1", Some(SkillId::Block)));

        let mut sm = SessionManager::new();
        let (tx, mut rx) = mpsc::unbounded_channel();
        sm.add_session(1, game_id, "Spec".into(), ClientMode::SPECTATOR, false, vec![], tx);

        let mut config = config_with_block_marking();
        assert!(h.handle_command(&mut config, game_id, 1, &cache, &sm));

        let sent = rx.try_recv().expect("expected a markers message");
        assert!(sent.contains("\"B\""));
    }

    #[test]
    fn handle_command_empty_config_fills_defaults() {
        let h = ServerCommandHandlerApplyAutomatedPlayerMarkings::new();
        let mut cache = GameCache::new();
        let game_id = cache.create_game_state();
        cache
            .get_game_state_by_id_mut(game_id)
            .unwrap()
            .start_game(team("home"), team("away"), Rules::Bb2025, 0);

        let mut sm = SessionManager::new();
        let (tx, _rx) = mpsc::unbounded_channel();
        sm.add_session(1, game_id, "Spec".into(), ClientMode::SPECTATOR, false, vec![], tx);

        let mut config = AutoMarkingConfig::new();
        assert!(config.get_markings().is_empty());
        h.handle_command(&mut config, game_id, 1, &cache, &sm);
        assert!(!config.get_markings().is_empty());
    }

    #[test]
    fn mark_for_player_home_coach_sets_home_text() {
        let h = ServerCommandHandlerApplyAutomatedPlayerMarkings::new();
        let game = game_with_marked_player();
        let config = config_with_block_marking();
        let mut sm = SessionManager::new();
        let (tx, mut rx) = mpsc::unbounded_channel();
        sm.add_session(1, 100, "Home".into(), ClientMode::PLAYER, true, vec![], tx);

        h.mark_for_player(&sm, &game, &config, 100, 1);

        let sent = rx.try_recv().expect("expected markers message");
        assert!(sent.contains("\"homeText\":\"B\""));
    }

    #[test]
    fn all_players_iterates_both_teams() {
        let mut home = team("home");
        home.players.push(player("p1", None));
        let mut away = team("away");
        away.players.push(player("p2", None));
        let game = Game::new(home, away, Rules::Bb2025);
        let ids: Vec<&str> = all_players(&game).map(|p| p.id.as_str()).collect();
        assert_eq!(ids, vec!["p1", "p2"]);
    }
}
