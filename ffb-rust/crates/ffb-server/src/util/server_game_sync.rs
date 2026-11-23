//! BACKLOG §P P3: what the clients see of the engine's game after a command.
//!
//! Java: `ServerCommunication.sendGameState(gameState)` (the whole game: home + spectators get
//! it as is, the away coach `transform()`ed) and `UtilServerGame.syncGameModel` ->
//! `sendModelSync` (the step's `ModelChangeList`, home + spectators as is, the away coach
//! `syncCommand.transform(rules)`). The Rust engine records no model changes, so the list is
//! derived by `model_change_diff::diff` from the game the clients last received
//! (`GameState::synced`) and proven by `self_check` before it is sent; a game the diff cannot
//! express yet goes out whole instead, so the clients always hold the engine's game.
use std::sync::Mutex;

use ffb_model::model::change::model_change_diff::{diff, list_to_json_value, self_check, transform_list};
use ffb_model::model::change::ModelChangeList;
use ffb_model::model::game::Game;
use ffb_protocol::commands::server_command_game_state::ServerCommandGameState;

use crate::game_state::GameState;
use crate::net::session_manager::SessionManager;

/// Java: `ServerCommunication.sendGameState(GameState)`.
pub fn send_game_state(game_state: &mut GameState, session_manager: &Mutex<SessionManager>) {
    let game_id = game_state.get_id();
    let game = game_state.get_game().cloned();
    let command = ServerCommandGameState::new(game.clone());
    {
        let sm = session_manager.lock().unwrap();
        sm.send_home_and_spectators(game_id, &command.to_json_value().to_string());
        sm.send_away(game_id, &command.transform().to_json_value().to_string());
    }
    game_state.synced = game;
}

/// Java: `ServerCommandModelSync.toJsonValue()` -- netCommandId, commandNr, modelChangeList,
/// reportList, (animation), sound, gameTime, turnTime.
fn model_sync_json(command_nr: i64, changes: &ModelChangeList, game: &Game) -> String {
    let mut o = serde_json::Map::new();
    o.insert("netCommandId".into(), "serverModelSync".into());
    o.insert("commandNr".into(), command_nr.into());
    o.insert("modelChangeList".into(), list_to_json_value(changes));
    // Reports are not sent yet (BACKLOG §P): the Java client decodes every report through its
    // ReportFactory, so only Java-shaped reports may go out.
    o.insert("reportList".into(), serde_json::json!({ "reports": [] }));
    o.insert("sound".into(), serde_json::Value::Null);
    o.insert("gameTime".into(), game.game_time.into());
    o.insert("turnTime".into(), game.turn_time.into());
    serde_json::Value::Object(o).to_string()
}

/// Prompts that Java answers without a dialog: the client acts from the turn mode, acting player
/// or the field's move/pushback squares. Every other prompt corresponds to a Java
/// `UtilServerDialog.showDialog`.
fn prompt_without_dialog(prompt: &ffb_model::prompts::AgentPrompt) -> bool {
    use ffb_model::prompts::AgentPrompt as P;
    matches!(
        prompt,
        P::ActivatePlayer { .. }
            | P::Move { .. }
            | P::TeamSetup { .. }
            | P::KickBall
            | P::Pushback { .. }
            | P::BlockTarget { .. }
            | P::ThrowTeamMateTarget { .. }
            | P::PuntTarget { .. }
            | P::SwoopTarget { .. }
            | P::HitAndRun { .. }
            | P::FuriousOutburstSquare { .. }
            | P::RaidingParty { .. }
            | P::BlastinTarget { .. }
            | P::TricksterMove { .. }
            | P::PlaceBallSquare { .. }
            | P::KickoffEventPlacement { .. }
            | P::BombRethrow { .. }
            | P::MultiBlockTargets { .. }
            | P::BlitzBlockSkill { .. }
            | P::StartGame
    )
}

/// After an engine command: send the model changes since the last sync, or the whole game when
/// they cannot be expressed (logged -- that log is the P3 worklist).
pub fn sync_game(game_state: &mut GameState, session_manager: &Mutex<SessionManager>) {
    if let (Some(prompt), Some(game)) = (
        game_state.driver.as_ref().and_then(|d| d.current_prompt()),
        game_state.get_game(),
    ) {
        if game.dialog_parameter.is_none() && !prompt_without_dialog(prompt) {
            log::warn!(
                "game {}: engine waits for {:?} but no Java dialog is set (P2 worklist)",
                game_state.get_id(),
                format!("{prompt:?}").chars().take(60).collect::<String>()
            );
        }
    }
    let Some(after) = game_state.get_game().cloned() else { return };
    let Some(before) = game_state.synced.clone() else {
        send_game_state(game_state, session_manager);
        return;
    };
    let list = match diff(&before, &after) {
        Ok(list) => list,
        Err(paths) => {
            log::warn!("game {}: no model change for {:?} -- sending the whole game", game_state.get_id(), paths);
            send_game_state(game_state, session_manager);
            return;
        }
    };
    log::debug!("game {}: diff -> {} changes", game_state.get_id(), list.len());
    if list.is_empty() {
        return;
    }
    let checked = self_check(&before, &after, &list)
        .and_then(|_| transform_list(&before, &list).map(|away| (list, away)));
    let (home, away) = match checked {
        Ok(lists) => lists,
        Err(e) => {
            log::warn!("game {}: model change self-check failed ({e}) -- sending the whole game", game_state.get_id());
            send_game_state(game_state, session_manager);
            return;
        }
    };
    let command_nr = game_state.generate_command_nr();
    let game_id = game_state.get_id();
    {
        let sm = session_manager.lock().unwrap();
        sm.send_home_and_spectators(game_id, &model_sync_json(command_nr, &home, &after));
        sm.send_away(game_id, &model_sync_json(command_nr, &away, &after));
    }
    game_state.synced = Some(after);
}
