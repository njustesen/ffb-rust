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
    let game = game_state.get_game().cloned();
    send_game(game_state, game, session_manager);
}

/// `send_game_state` of a given game (the live one, or a step's copy -- see `sync_game`).
fn send_game(game_state: &mut GameState, game: Option<Game>, session_manager: &Mutex<SessionManager>) {
    let game_id = game_state.get_id();
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
fn model_sync_json(command_nr: i64, changes: &ModelChangeList, reports: &[serde_json::Value], game: &Game) -> String {
    let mut o = serde_json::Map::new();
    o.insert("netCommandId".into(), "serverModelSync".into());
    o.insert("commandNr".into(), command_nr.into());
    o.insert("modelChangeList".into(), list_to_json_value(changes));
    // ReportList.toJsonValue: {"reports": [IReport.toJsonValue ...]}.
    o.insert("reportList".into(), serde_json::json!({ "reports": reports }));
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

/// The engine's reports added since the last sync, as Java JSON (the same `to_json` the report
/// parity compares with Java's stream; a report without a Java form is not sent -- the client's
/// ReportFactory could not decode it). Every writer the parity suites reach decodes in the Java
/// client (`ReportWireCheck`, BACKLOG §P); `FFB_SERVER_REPORTS=0` turns the game log off.
fn new_reports(game_state: &mut GameState, upto: Option<usize>) -> Vec<serde_json::Value> {
    if std::env::var_os("FFB_SERVER_REPORTS").is_some_and(|v| v == "0") {
        return Vec::new();
    }
    // The live engine game: a cloned `Game` carries an EMPTY report list (`ReportList::clone`).
    let (out, total, start) = match game_state.get_game() {
        Some(live) => {
            let reports = live.report_list.get_reports();
            let end = upto.unwrap_or(reports.len()).min(reports.len());
            let start = game_state.reports_sent.min(end);
            let out: Vec<serde_json::Value> =
                reports[start..end].iter().filter_map(|r| r.to_json()).map(client_form).collect();
            (out, end, start)
        }
        None => return Vec::new(),
    };
    if total > start {
        log::debug!("game {}: {} reports -> {} sent", game_state.get_id(), total - start, out.len());
    }
    game_state.reports_sent = total;
    out
}

/// The report as the client receives it. A `null` report field goes out as an absent key: for
/// every Java `IJsonOption.getFrom` an absent key reads as Java null, while a JSON null is
/// rejected by several (`JsonBooleanOption` -> `asBoolean`, `UtilJson.toJsonObject`), so dropping
/// it never changes what a client decodes and keeps a writer that is not yet checked against
/// `ReportWireCheck` from dropping a whole sync.
fn client_form(v: serde_json::Value) -> serde_json::Value {
    match v {
        serde_json::Value::Object(o) => {
            serde_json::Value::Object(o.into_iter().filter(|(_, v)| !v.is_null()).collect())
        }
        other => other,
    }
}

/// Java `ReportList.transform(source)` -> `IReport.transform(source)` for the away coach, on the
/// Java wire form of each report. Every non-identity Java `transform` (survey of
/// `ffb-common/.../report/**`): coordinates `FieldCoordinate.transform`, directions
/// `Direction.transform` (singly or `DirectionFactory.transform(Direction[])`), and the reports
/// whose constructor is called with the home and away values exchanged.
fn transform_report(mut v: serde_json::Value) -> serde_json::Value {
    use ffb_model::enums::Direction;
    use ffb_model::types::FieldCoordinate;
    use serde_json::Value;
    let Some(o) = v.as_object_mut() else { return v };
    let id = o.get("reportId").and_then(Value::as_str).unwrap_or("").to_string();
    let coordinate = |x: &mut Value| {
        if let Some([cx, cy]) = x.as_array().map(Vec::as_slice) {
            if let (Some(cx), Some(cy)) = (cx.as_i64(), cy.as_i64()) {
                let t = FieldCoordinate::new(cx as i32, cy as i32).transform();
                *x = serde_json::json!([t.x, t.y]);
            }
        }
    };
    let direction = |x: &mut Value| {
        if let Some(d) = x.as_str().and_then(Direction::from_name) {
            *x = Value::String(d.transform().name().to_string());
        }
    };
    let (coordinates, directions): (&[&str], &[&str]) = match id.as_str() {
        // ReportKickoffScatter / ReportPassDeviate
        "kickoffScatter" | "passDeviate" => (&["ballCoordinateEnd"], &["scatterDirection"]),
        // ReportScatterPlayer, bb2016/2020/2025 ReportSwoopPlayer (direction or directionArray)
        "scatterPlayer" | "swoopPlayer" => (&["startCoordinate", "endCoordinate"], &["directionArray", "scatterDirection", "direction"]),
        "scatterBall" => (&[], &["directionArray"]),
        // ReportThrowIn, ReportPuntDirection, ReportSwoopDirection, ReportHitAndRun,
        // ReportPlaceBallDirection, ReportRaidingParty
        "throwIn" | "puntDirection" | "swoopDirection" | "hitAndRun" | "placeBallDirection" | "raidingParty" => (&[], &["direction"]),
        _ => (&[], &[]),
    };
    for k in coordinates {
        if let Some(x) = o.get_mut(*k) {
            coordinate(x);
        }
    }
    for k in directions {
        match o.get_mut(*k) {
            Some(Value::Array(a)) => a.iter_mut().for_each(direction),
            Some(x) => direction(x),
            None => {}
        }
    }
    // The reports whose transform swaps the home and away values (bb2016 FanFactorRollPostMatch,
    // KickoffExtraReRoll, KickoffPitchInvasion, KickoffThrowARock, PenaltyShootout, Spectators,
    // WinningsRoll; bb2020/bb2025 CheeringFans; KickoffOfficiousRef, KickoffDodgySnack;
    // mixed DedicatedFans, KickoffExtraReRoll, KickoffPitchInvasion, PenaltyShootout, Winnings;
    // MostValuablePlayers): every `<x>Home` key exchanges its value with `<x>Away`.
    const SWAPS: &[&str] = &[
        "fanFactorRollPostMatch", "kickoffExtraReRoll", "kickoffPitchInvasion", "kickoffThrowARock",
        "penaltyShootout", "spectators", "winningsRoll", "cheeringFans", "kickoffOfficiousRef",
        "kickoffDodgySnack", "dedicatedFans", "winnings", "mostValuablePlayers",
    ];
    if SWAPS.contains(&id.as_str()) {
        // `<x>Home` / `<x>Away`, and the bb2016 kick-off re-roll's `homeGainsReRoll` /
        // `awayGainsReRoll` (`homeTeam` is the penalty winner flag, negated below, not swapped).
        let homes: Vec<(String, String)> = o
            .keys()
            .filter_map(|k| {
                if let Some(p) = k.strip_suffix("Home") {
                    Some((k.clone(), format!("{p}Away")))
                } else if let Some(r) = k.strip_prefix("home").filter(|r| *r != "Team" && r.starts_with(char::is_uppercase)) {
                    Some((k.clone(), format!("away{r}")))
                } else {
                    None
                }
            })
            .collect();
        for (home, away) in homes {
            let h = o.remove(&home);
            let a = o.remove(&away);
            if let Some(a) = a {
                o.insert(home.clone(), a);
            }
            if let Some(h) = h {
                o.insert(away, h);
            }
        }
    }
    // mixed ReportPenaltyShootout: `homeTeamWonPenalty == null ? null : !homeTeamWonPenalty`.
    // mixed ReportPrayerAmount: `new ReportPrayerAmount(tvAway, tvHome, amount, !homeTeam)`.
    if id == "penaltyShootout" || id == "prayerAmount" {
        if let Some(Value::Bool(b)) = o.get_mut("homeTeam") {
            *b = !*b;
        }
    }
    if id == "prayerAmount" {
        let tv = o.remove("teamValue");
        let otv = o.remove("opponentTeamValue");
        if let Some(x) = otv {
            o.insert("teamValue".into(), x);
        }
        if let Some(x) = tv {
            o.insert("opponentTeamValue".into(), x);
        }
    }
    v
}

/// A sync that only carries reports (after a whole-game send, which has none).
fn send_reports(game_state: &mut GameState, upto: Option<usize>, game: &Game, session_manager: &Mutex<SessionManager>) {
    let reports = new_reports(game_state, upto);
    if reports.is_empty() {
        return;
    }
    let away_reports: Vec<serde_json::Value> = reports.iter().cloned().map(transform_report).collect();
    let command_nr = game_state.generate_command_nr();
    let empty = ModelChangeList::default();
    let sm = session_manager.lock().unwrap();
    sm.send_home_and_spectators(game_state.get_id(), &model_sync_json(command_nr, &empty, &reports, game));
    sm.send_away(game_state.get_id(), &model_sync_json(command_nr, &empty, &away_reports, game));
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
    // Java syncs after every step: first the copies of the game after each step that reported
    // (`DriverGameState::begin_report_snapshots`), each with the reports up to that step, then
    // the game as the command left it.
    let snapshots = game_state.driver.as_mut().map(|d| d.take_report_snapshots()).unwrap_or_default();
    for (game, reports) in snapshots {
        sync_to(game_state, game, Some(reports), session_manager);
    }
    if let Some(after) = game_state.get_game().cloned() {
        sync_to(game_state, after, None, session_manager);
    }
}

/// One `serverModelSync` from the game the clients hold to `after`, with the engine's reports up
/// to `upto` (all of them when `None`).
fn sync_to(game_state: &mut GameState, after: Game, upto: Option<usize>, session_manager: &Mutex<SessionManager>) {
    let Some(before) = game_state.synced.clone() else {
        send_game(game_state, Some(after), session_manager);
        return;
    };
    let list = match diff(&before, &after) {
        Ok(list) => list,
        Err(paths) => {
            log::warn!("game {}: no model change for {:?} -- sending the whole game", game_state.get_id(), paths);
            send_game(game_state, Some(after.clone()), session_manager);
            send_reports(game_state, upto, &after, session_manager);
            return;
        }
    };
    log::debug!("game {}: diff -> {} changes", game_state.get_id(), list.len());
    if list.is_empty() {
        send_reports(game_state, upto, &after, session_manager);
        return;
    }
    let checked = self_check(&before, &after, &list)
        .and_then(|_| transform_list(&before, &list).map(|away| (list, away)));
    let (home, away) = match checked {
        Ok(lists) => lists,
        Err(e) => {
            log::warn!("game {}: model change self-check failed ({e}) -- sending the whole game", game_state.get_id());
            send_game(game_state, Some(after.clone()), session_manager);
            send_reports(game_state, upto, &after, session_manager);
            return;
        }
    };
    let reports = new_reports(game_state, upto);
    // Java: the away coach gets `syncCommand.transform(rules)` -- its ReportList transformed too.
    let away_reports: Vec<serde_json::Value> = reports.iter().cloned().map(transform_report).collect();
    let command_nr = game_state.generate_command_nr();
    let game_id = game_state.get_id();
    {
        let sm = session_manager.lock().unwrap();
        sm.send_home_and_spectators(game_id, &model_sync_json(command_nr, &home, &reports, &after));
        sm.send_away(game_id, &model_sync_json(command_nr, &away, &away_reports, &after));
    }
    game_state.synced = Some(after);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn away_reports_mirror_coordinates_and_directions() {
        let r = transform_report(json!({"reportId": "kickoffScatter", "ballCoordinateEnd": [4, 9],
            "scatterDirection": "Southwest", "rollScatterDirection": 6, "rollScatterDistance": 2}));
        assert_eq!(r["ballCoordinateEnd"], json!([21, 9]));
        assert_eq!(r["scatterDirection"], json!("Southeast"));
        let r = transform_report(json!({"reportId": "scatterBall", "directionArray": ["West", "North"]}));
        assert_eq!(r["directionArray"], json!(["East", "North"]));
        // a report Java does not transform is unchanged
        let r = json!({"reportId": "goForItRoll", "playerId": "p", "roll": 2});
        assert_eq!(transform_report(r.clone()), r);
    }

    #[test]
    fn away_reports_swap_home_and_away() {
        let r = transform_report(json!({"reportId": "kickoffExtraReRoll", "rollHome": 5, "homeGainsReRoll": true,
            "rollAway": 2, "awayGainsReRoll": false}));
        assert_eq!(r, json!({"reportId": "kickoffExtraReRoll", "rollHome": 2, "homeGainsReRoll": false,
            "rollAway": 5, "awayGainsReRoll": true}));
        let r = transform_report(json!({"reportId": "penaltyShootout", "rollHome": 3, "rollAway": 4,
            "homeTeam": true, "penaltyScoreHome": 1, "penaltyScoreAway": 2}));
        assert_eq!(r["homeTeam"], json!(false));
        assert_eq!(r["penaltyScoreHome"], json!(2));
        let r = transform_report(json!({"reportId": "prayerAmount", "teamValue": 1, "opponentTeamValue": 2,
            "homeTeam": false, "number": 3}));
        assert_eq!(r, json!({"reportId": "prayerAmount", "teamValue": 2, "opponentTeamValue": 1,
            "homeTeam": true, "number": 3}));
    }

    #[test]
    fn client_form_drops_null_fields() {
        let r = client_form(json!({"reportId": "scatterPlayer", "isScatter": null, "rolls": [1]}));
        assert_eq!(r, json!({"reportId": "scatterPlayer", "rolls": [1]}));
    }
}
