//! BACKLOG §P P3: the `ModelChange`s a Java server would have sent for one engine command.
//!
//! Java records model changes as the steps call the model setters (`notifyObservers` ->
//! `ModelChangeList`). The Rust engine mutates its `Game` directly, so the server derives the list
//! afterwards: `diff(before, after)` compares the two games in their Java-JSON form
//! (`Game::to_json_value`) and emits, per changed value, the `ModelChange` whose
//! `ModelChangeProcessor.apply` sets exactly that value -- the processor is the inverse spec of
//! every table below. A changed value no table covers is an `Err` naming its JSON path: the caller
//! then sends the whole game instead (`serverGameState`), and `self_check` proves every list it
//! does send by applying it to `before` and comparing the result with `after`.
//!
//! This is server plumbing with no single Java counterpart; each table entry cites the setter that
//! emits the change in Java (`Game.setX` -> `GAME_SET_X`, ...).

use super::model_change::ModelChange;
use super::model_change_list::ModelChangeList;
use super::model_change_processor::{DecodedModelChange, ModelChangeProcessor};
use crate::enums::ModelChangeId;
use crate::model::game::Game;
use serde_json::{Map, Value};

/// `Game` scalars (Java `Game.setX`). `None` = a value with no model change (the client never
/// receives it as a change); keyed ids carry the value in the key (`gameSetDefenderId` & co.).
const GAME_KEYS: &[(&str, Option<&str>)] = &[
    ("gameId", Some("gameSetId")),
    ("scheduled", Some("gameSetScheduled")),
    ("started", Some("gameSetStarted")),
    ("finished", Some("gameSetFinished")),
    ("homePlaying", Some("gameSetHomePlaying")),
    ("half", Some("gameSetHalf")),
    ("homeFirstOffense", Some("gameSetHomeFirstOffense")),
    ("setupOffense", Some("gameSetSetupOffense")),
    ("waitingForOpponent", Some("gameSetWaitingForOpponent")),
    ("timeoutPossible", Some("gameSetTimeoutPossible")),
    ("timeoutEnforced", Some("gameSetTimeoutEnforced")),
    ("concessionPossible", Some("gameSetConcessionPossible")),
    ("testing", Some("gameSetTesting")),
    ("turnMode", Some("gameSetTurnMode")),
    ("lastTurnMode", Some("gameSetLastTurnMode")),
    ("defenderAction", Some("gameSetDefenderAction")),
    ("passCoordinate", Some("gameSetPassCoordinate")),
    ("throwerAction", Some("gameSetThrowerAction")),
    ("concededLegally", Some("gameSetConcededLegally")),
    ("dialogParameter", Some("gameSetDialogParameter")),
];

/// Keyed `Game` ids: Java `setDefenderId(id)` notifies `(GAME_SET_DEFENDER_ID, id, null)`.
const GAME_KEYED: &[(&str, &str)] = &[
    ("defenderId", "gameSetDefenderId"),
    ("lastDefenderId", "gameSetLastDefenderId"),
    ("throwerId", "gameSetThrowerId"),
];

/// `TurnData.setX`, keyed `home` / `away`.
const TURN_DATA_KEYS: &[(&str, &str)] = &[
    ("turnStarted", "turnDataSetTurnStarted"),
    ("turnNr", "turnDataSetTurnNr"),
    ("firstTurnAfterKickoff", "turnDataSetFirstTurnAfterKickoff"),
    ("reRolls", "turnDataSetReRolls"),
    ("rerollBrilliantCoachingOneDrive", "turnDataSetReRollsBrilliantCoachingOneDrive"),
    ("rerollPumpUpTheCrowdOneDrive", "turnDataSetReRollsPumpUpTheCrowdOneDrive"),
    ("apothecaries", "turnDataSetApothecaries"),
    ("blitzUsed", "turnDataSetBlitzUsed"),
    ("foulUsed", "turnDataSetFoulUsed"),
    ("reRollUsed", "turnDataSetReRollUsed"),
    ("handOverUsed", "turnDataSetHandOverUsed"),
    ("passUsed", "turnDataSetPassUsed"),
    ("coachBanned", "turnDataSetCoachBanned"),
    ("ktmUsed", "turnDataSetKtmUsed"),
    ("bombUsed", "turnDataSetBombUsed"),
    ("secureTheBallUsed", "turnDataSecureTheBallUsed"),
    ("leaderState", "turnDataSetLeaderState"),
    ("wanderingApothecaries", "turnDataSetWanderingApothecaries"),
    ("plagueDoctors", "turnDataSetPlagueDoctors"),
    ("ttmUsed", "turnDataSetTtmUsed"),
    ("puntUsed", "turnDataPuntUsed"),
];

/// `ActingPlayer.setX` (unkeyed). `playerId` first: Java `setPlayerId` resets the rest.
const ACTING_PLAYER_KEYS: &[(&str, &str)] = &[
    ("currentMove", "actingPlayerSetCurrentMove"),
    ("goingForIt", "actingPlayerSetGoingForIt"),
    ("hasBlocked", "actingPlayerSetHasBlocked"),
    ("hasFed", "actingPlayerSetHasFed"),
    ("hasFouled", "actingPlayerSetHasFouled"),
    ("hasMoved", "actingPlayerSetHasMoved"),
    ("hasPassed", "actingPlayerSetHasPassed"),
    ("playerAction", "actingPlayerSetPlayerAction"),
    ("standingUp", "actingPlayerSetStandingUp"),
    ("sufferingAnimosity", "actingPlayerSetSufferingAnimosity"),
    ("sufferingBloodlust", "actingPlayerSetSufferingBloodLust"),
    ("jumpsWithoutModifiers", "actingPlayerSetJumpsWithoutModifiers"),
    ("heldInPlace", "actingPlayerSetHeldInPlace"),
    ("mustCompleteAction", "actingPlayerSetMustCompleteAction"),
    ("fellFromRush", "actingPlayerSetFellFromRush"),
    ("hasTriggeredEffect", "actingPlayerSetHasTriggeredEffect"),
];

/// `FieldModel` scalars (unkeyed).
const FIELD_MODEL_KEYS: &[(&str, &str)] = &[
    ("weather", "fieldModelSetWeather"),
    ("ballCoordinate", "fieldModelSetBallCoordinate"),
    ("ballInPlay", "fieldModelSetBallInPlay"),
    ("ballMoving", "fieldModelSetBallMoving"),
    ("bombCoordinate", "fieldModelSetBombCoordinate"),
    ("bombMoving", "fieldModelSetBombMoving"),
    ("outOfBounds", "fieldModelOutOfBounds"),
    ("rangeRuler", "fieldModelSetRangeRuler"),
    ("targetSelectionState", "fieldModelSetTargetSelectionState"),
];

/// `FieldModel` element sets: (array key, add id, remove id). Elements are compared whole.
const FIELD_MODEL_SETS: &[(&str, &str, Option<&str>)] = &[
    ("bloodspotArray", "fieldModelAddBloodSpot", None),
    ("pushbackSquareArray", "fieldModelAddPushbackSquare", Some("fieldModelRemovePushbackSquare")),
    ("moveSquareArray", "fieldModelAddMoveSquare", Some("fieldModelRemoveMoveSquare")),
    ("trackNumberArray", "fieldModelAddTrackNumber", Some("fieldModelRemoveTrackNumber")),
    ("diceDecorationArray", "fieldModelAddDiceDecoration", Some("fieldModelRemoveDiceDecoration")),
    ("fieldMarkerArray", "fieldModelAddFieldMarker", Some("fieldModelRemoveFieldMarker")),
    ("playerMarkerArray", "fieldModelAddPlayerMarker", Some("fieldModelRemovePlayerMarker")),
    ("trapDoors", "fieldModelAddTrapDoor", Some("fieldModelRemoveTrapDoor")),
];

/// `TeamResult.setX`, keyed `home` / `away`.
const TEAM_RESULT_KEYS: &[(&str, &str)] = &[
    ("score", "teamResultSetScore"),
    ("conceded", "teamResultSetConceded"),
    ("raisedDead", "teamResultSetRaisedDead"),
    ("spectators", "teamResultSetSpectators"),
    ("fame", "teamResultSetFame"),
    ("winnings", "teamResultSetWinnings"),
    ("fanFactorModifier", "teamResultSetFanFactorModifier"),
    ("badlyHurtSuffered", "teamResultSetBadlyHurtSuffered"),
    ("seriousInjurySuffered", "teamResultSetSeriousInjurySuffered"),
    ("ripSuffered", "teamResultSetRipSuffered"),
    ("spirallingExpenses", "teamResultSetSpirallingExpenses"),
    ("pettyCashTransferred", "teamResultSetPettyCashTransferred"),
    ("pettyCashUsed", "teamResultSetPettyCashUsed"),
    ("teamValue", "teamResultSetTeamValue"),
    ("fanFactor", "teamResultSetFanFactor"),
    ("dedicatedFans", "teamResultDedicatedFansModifier"),
    ("penaltyScore", "teamResultSetPenaltyScore"),
];

/// `PlayerResult.setX`, keyed by player id.
const PLAYER_RESULT_KEYS: &[(&str, &str)] = &[
    ("completions", "playerResultSetCompletions"),
    ("completionsWithAdditionalSpp", "playerResultSetCompletionsWithAdditionalSpp"),
    ("touchdowns", "playerResultSetTouchdowns"),
    ("interceptions", "playerResultSetInterceptions"),
    ("deflections", "playerResultSetDeflections"),
    ("casualties", "playerResultSetCasualties"),
    ("casualtiesWithAdditionalSpp", "playerResultSetCasualtiesWithAdditionalSpp"),
    ("playerAwards", "playerResultSetPlayerAwards"),
    ("blocks", "playerResultSetBlocks"),
    ("fouls", "playerResultSetFouls"),
    ("rushing", "playerResultSetRushing"),
    ("passing", "playerResultSetPassing"),
    ("currentSpps", "playerResultSetCurrentSpps"),
    ("seriousInjury", "playerResultSetSeriousInjury"),
    ("seriousInjuryDecay", "playerResultSetSeriousInjuryDecay"),
    ("sendToBoxReason", "playerResultSetSendToBoxReason"),
    ("sendToBoxTurn", "playerResultSetSendToBoxTurn"),
    ("sendToBoxHalf", "playerResultSetSendToBoxHalf"),
    ("sendToBoxByPlayerId", "playerResultSetSendToBoxByPlayerId"),
    ("turnsPlayed", "playerResultSetTurnsPlayed"),
    ("hasUsedSecretWeapon", "playerResultSetHasUsedSecretWeapon"),
    ("defecting", "playerResultSetDefecting"),
    ("catchesWithAdditionalSpp", "playerResultSetCatchesWithAdditionalSpp"),
    ("landings", "playerResultSetLandings"),
];

fn change(name: &str, key: Option<&str>, value: Value) -> ModelChange {
    let id = ModelChangeId::for_name(name).unwrap_or_else(|| panic!("model change id {name:?}"));
    ModelChange::with_key(id, key.map(str::to_string), value)
}

fn obj<'a>(v: &'a Value, path: &str) -> Result<&'a Map<String, Value>, String> {
    v.as_object().ok_or_else(|| format!("{path}: not an object"))
}

fn get<'a>(o: &'a Map<String, Value>, k: &str) -> &'a Value {
    o.get(k).unwrap_or(&Value::Null)
}

/// Collects the changes, and every changed path no table covers.
struct Differ {
    list: ModelChangeList,
    uncovered: Vec<String>,
    /// Java would have notified GAME_SET_DIALOG_PARAMETER (see `Game::dialog_set_count`).
    dialog_set: bool,
}

impl Differ {
    /// Compare two objects key by key through `table`; keys in `skip` are handled elsewhere, any
    /// other differing key is uncovered.
    fn scalars(&mut self, path: &str, b: &Map<String, Value>, a: &Map<String, Value>, table: &[(&str, &str)], key: Option<&str>, skip: &[&str]) {
        let mut keys: Vec<&String> = b.keys().chain(a.keys()).collect();
        keys.sort();
        keys.dedup();
        for k in keys {
            if skip.contains(&k.as_str()) {
                continue;
            }
            let (bv, av) = (get(b, k), get(a, k));
            if bv == av {
                continue;
            }
            match table.iter().find(|(json_key, _)| json_key == k) {
                Some((_, id)) => self.list.add(change(id, key, av.clone())),
                None => self.uncovered.push(format!("{path}.{k}")),
            }
        }
    }

    fn game(&mut self, b: &Value, a: &Value) -> Result<(), String> {
        let (b, a) = (obj(b, "game")?, obj(a, "game")?);
        // turnTime / gameTime travel in the sync command itself; teamState is always FULL.
        let handled = [
            "turnTime", "gameTime", "teamState", "teamHome", "teamAway", "turnDataHome", "turnDataAway",
            "fieldModel", "actingPlayer", "gameResult", "gameOptions", "defenderId", "lastDefenderId", "throwerId",
            "turnMode", "lastTurnMode",
        ];
        // GAME_KEYS has Option ids; flatten the covered ones.
        let table: Vec<(&str, &str)> = GAME_KEYS.iter().filter_map(|(k, id)| id.map(|id| (*k, id))).collect();
        let mut handled = handled.to_vec();
        handled.push("dialogParameter");
        self.scalars("game", b, a, &table, None, &handled);
        if self.dialog_set || get(b, "dialogParameter") != get(a, "dialogParameter") {
            self.list.add(change("gameSetDialogParameter", None, get(a, "dialogParameter").clone()));
        }
        // Java `setTurnMode` moves the old mode into lastTurnMode itself, so after a turn-mode
        // change the engine's lastTurnMode is set again explicitly.
        let turn_mode_changed = get(b, "turnMode") != get(a, "turnMode");
        if turn_mode_changed {
            self.list.add(change("gameSetTurnMode", None, get(a, "turnMode").clone()));
        }
        if turn_mode_changed || get(b, "lastTurnMode") != get(a, "lastTurnMode") {
            self.list.add(change("gameSetLastTurnMode", None, get(a, "lastTurnMode").clone()));
        }
        for (k, id) in GAME_KEYED {
            let (bv, av) = (get(b, k), get(a, k));
            if bv != av {
                self.list.add(change(id, av.as_str(), Value::Null));
            }
        }
        for (side, td, team) in [("home", "turnDataHome", "teamHome"), ("away", "turnDataAway", "teamAway")] {
            let (bt, at) = (obj(get(b, td), td)?, obj(get(a, td), td)?);
            self.scalars(td, bt, at, TURN_DATA_KEYS, Some(side), &["homeData", "inducementSet"]);
            if get(bt, "inducementSet") != get(at, "inducementSet") {
                self.inducement_set(td, side, obj(get(bt, "inducementSet"), td)?, obj(get(at, "inducementSet"), td)?);
            }
            if get(b, team) != get(a, team) {
                self.team(team, obj(get(b, team), team)?, obj(get(a, team), team)?)?;
            }
        }
        self.acting_player(obj(get(b, "actingPlayer"), "actingPlayer")?, obj(get(a, "actingPlayer"), "actingPlayer")?);
        self.field_model(obj(get(b, "fieldModel"), "fieldModel")?, obj(get(a, "fieldModel"), "fieldModel")?)?;
        self.game_result(obj(get(b, "gameResult"), "gameResult")?, obj(get(a, "gameResult"), "gameResult")?)?;
        if get(b, "gameOptions") != get(a, "gameOptions") {
            self.uncovered.push("gameOptions".into());
        }
        Ok(())
    }

    /// `InducementSet.addInducement` / `removeInducement` / `addPrayer` / `removePrayer`, keyed
    /// home / away; cards are not covered yet.
    fn inducement_set(&mut self, path: &str, side: &str, b: &Map<String, Value>, a: &Map<String, Value>) {
        for k in ["cardsAvailable", "cardsActive", "cardsDeactivated", "starPlayerPositionIds"] {
            if get(b, k) != get(a, k) {
                self.uncovered.push(format!("{path}.inducementSet.{k}"));
            }
        }
        let empty = Vec::new();
        let by_type = |o: &Map<String, Value>| -> Vec<(String, Value)> {
            get(o, "inducementArray").as_array().unwrap_or(&empty).iter()
                .map(|i| (i.get("inducementType").and_then(Value::as_str).unwrap_or("").to_string(), i.clone()))
                .collect()
        };
        let (bi, ai) = (by_type(b), by_type(a));
        for (t, v) in &bi {
            if !ai.iter().any(|(at, _)| at == t) {
                self.list.add(change("inducementSetRemoveInducement", Some(side), v.clone()));
            }
        }
        for (t, v) in &ai {
            if bi.iter().find(|(bt, _)| bt == t).map(|(_, bv)| bv) != Some(v) {
                self.list.add(change("inducementSetAddInducement", Some(side), v.clone()));
            }
        }
        let set = |o: &Map<String, Value>| get(o, "prayers").as_array().cloned().unwrap_or_default();
        let (bp, ap) = (set(b), set(a));
        for p in bp.iter().filter(|p| !ap.contains(p)) {
            self.list.add(change("inducementSetRemovePrayer", Some(side), p.clone()));
        }
        for p in ap.iter().filter(|p| !bp.contains(p)) {
            self.list.add(change("inducementSetAddPrayer", Some(side), p.clone()));
        }
    }

    /// A team: only a player's used skills travel as model changes (Java `RosterPlayer.markUsed`
    /// / `markUnused` -> PLAYER_MARK_SKILL_USED / UNUSED, keyed by the player id).
    fn team(&mut self, path: &str, b: &Map<String, Value>, a: &Map<String, Value>) -> Result<(), String> {
        let mut keys: Vec<&String> = b.keys().chain(a.keys()).collect();
        keys.sort();
        keys.dedup();
        for k in keys {
            if k != "playerArray" && get(b, k) != get(a, k) {
                self.uncovered.push(format!("{path}.{k}"));
            }
        }
        let empty = Vec::new();
        let bp = get(b, "playerArray").as_array().unwrap_or(&empty);
        let ap = get(a, "playerArray").as_array().unwrap_or(&empty);
        if bp.len() != ap.len() {
            self.uncovered.push(format!("{path}.playerArray (players added/removed)"));
            return Ok(());
        }
        for (be, ae) in bp.iter().zip(ap) {
            if be == ae {
                continue;
            }
            let (bo, ao) = (obj(be, "player")?, obj(ae, "player")?);
            let id = get(ao, "playerId").as_str().unwrap_or("").to_string();
            if get(bo, "playerId") != get(ao, "playerId") {
                self.uncovered.push(format!("{path}.playerArray (order)"));
                continue;
            }
            for k in ao.keys().chain(bo.keys()) {
                if k != "usedSkills" && get(bo, k) != get(ao, k) {
                    self.uncovered.push(format!("{path}.player[{id}].{k}"));
                }
            }
            let set = |v: &Value| v.as_array().cloned().unwrap_or_default();
            let (bu, au) = (set(get(bo, "usedSkills")), set(get(ao, "usedSkills")));
            for sk in au.iter().filter(|x| !bu.contains(x)) {
                self.list.add(change("playerMarkSkillUsed", Some(&id), sk.clone()));
            }
            for sk in bu.iter().filter(|x| !au.contains(x)) {
                self.list.add(change("playerMarkSkillUnused", Some(&id), sk.clone()));
            }
        }
        Ok(())
    }

    fn acting_player(&mut self, b: &Map<String, Value>, a: &Map<String, Value>) {
        let id_changed = get(b, "playerId") != get(a, "playerId");
        if id_changed {
            // Java `setPlayerId` resets the acting player; every value is then set again.
            self.list.add(change("actingPlayerSetPlayerId", None, get(a, "playerId").clone()));
            let empty = Map::new();
            self.scalars("actingPlayer", &empty, a, ACTING_PLAYER_KEYS, None, &["playerId", "usedSkills", "skillsGrantedBy", "fumblerooskiePending", "playerStateOld"]);
        } else {
            self.scalars("actingPlayer", b, a, ACTING_PLAYER_KEYS, None, &["playerId", "usedSkills", "skillsGrantedBy", "playerStateOld"]);
        }
        let as_set = |v: &Value| -> Vec<Value> { v.as_array().cloned().unwrap_or_default() };
        let (bu, au) = (as_set(get(b, "usedSkills")), as_set(get(a, "usedSkills")));
        // setPlayerId clears the used skills, so a new acting player re-marks every one.
        if id_changed || bu != au {
            let base = if id_changed { Vec::new() } else { bu };
            for s in au.iter().filter(|s| !base.contains(s)) {
                self.list.add(change("actingPlayerMarkSkillUsed", None, s.clone()));
            }
            for s in base.iter().filter(|s| !au.contains(s)) {
                self.list.add(change("actingPlayerMarkSkillUnused", None, s.clone()));
            }
        }
        // Java `setOldPlayerState` ignores a non-null value over a non-null one: clear it first.
        let (bo, ao) = (get(b, "playerStateOld"), get(a, "playerStateOld"));
        if id_changed || bo != ao {
            if !ao.is_null() && (id_changed || !bo.is_null()) {
                self.list.add(change("actingPlayerSetOldPlayerState", None, Value::Null));
            }
            if !(ao.is_null() && bo.is_null() && !id_changed) {
                self.list.add(change("actingPlayerSetOldPlayerState", None, ao.clone()));
            }
        }
        if get(b, "skillsGrantedBy") != get(a, "skillsGrantedBy") {
            self.uncovered.push("actingPlayer.skillsGrantedBy".into());
        }
        if id_changed && get(a, "fumblerooskiePending") != &Value::Bool(false) || !id_changed && get(b, "fumblerooskiePending") != get(a, "fumblerooskiePending") {
            self.uncovered.push("actingPlayer.fumblerooskiePending".into());
        }
    }

    fn field_model(&mut self, b: &Map<String, Value>, a: &Map<String, Value>) -> Result<(), String> {
        let sets: Vec<&str> = FIELD_MODEL_SETS.iter().map(|(k, _, _)| *k).collect();
        let mut skip = vec!["playerDataArray", "chomped"];
        skip.extend(sets.iter());
        self.scalars("fieldModel", b, a, FIELD_MODEL_KEYS, None, &skip);
        for (k, add, remove) in FIELD_MODEL_SETS {
            let empty = Vec::new();
            let bs = get(b, k).as_array().unwrap_or(&empty);
            let as_ = get(a, k).as_array().unwrap_or(&empty);
            if remove.is_none() {
                // A Java List (blood spots): add-only, duplicates allowed -- a multiset difference.
                let mut rest: Vec<&Value> = bs.iter().collect();
                for e in as_ {
                    match rest.iter().position(|x| *x == e) {
                        Some(pos) => {
                            rest.remove(pos);
                        }
                        None => self.list.add(change(add, None, e.clone())),
                    }
                }
                if !rest.is_empty() {
                    self.uncovered.push(format!("fieldModel.{k} (removed)"));
                }
                continue;
            }
            for e in bs.iter().filter(|e| !as_.contains(e)) {
                if let Some(id) = remove {
                    self.list.add(change(id, None, e.clone()));
                }
            }
            for e in as_.iter().filter(|e| !bs.contains(e)) {
                self.list.add(change(add, None, e.clone()));
            }
        }
        if get(b, "chomped") != get(a, "chomped") {
            self.uncovered.push("fieldModel.chomped".into());
        }
        // playerDataArray: one entry per player of both teams.
        let index = |v: &Value| -> Result<Vec<(String, Map<String, Value>)>, String> {
            let mut out = Vec::new();
            for e in v.as_array().ok_or("fieldModel.playerDataArray: not an array")? {
                let o = obj(e, "playerData")?;
                let id = get(o, "playerId").as_str().ok_or("playerData.playerId")?.to_string();
                out.push((id, o.clone()));
            }
            Ok(out)
        };
        let (bp, ap) = (index(get(b, "playerDataArray"))?, index(get(a, "playerDataArray"))?);
        for (id, ao) in &ap {
            let empty = Map::new();
            let bo = bp.iter().find(|(bid, _)| bid == id).map(|(_, o)| o).unwrap_or(&empty);
            // state before coordinate: Java's setup / kick-off steps set the state of a player
            // entering the field first; the end state is the same either way.
            if get(bo, "playerState") != get(ao, "playerState") {
                self.list.add(change("fieldModelSetPlayerState", Some(id), get(ao, "playerState").clone()));
            }
            if get(bo, "playerCoordinate") != get(ao, "playerCoordinate") {
                // Java `setPlayerCoordinate(player, null)` is a no-op; leaving the pitch entirely is
                // `FieldModel.remove(player)` -> FIELD_MODEL_REMOVE_PLAYER.
                if get(ao, "playerCoordinate").is_null() {
                    self.list.add(change("fieldModelRemovePlayer", Some(id), Value::Null));
                } else {
                    self.list.add(change("fieldModelSetPlayerCoordinate", Some(id), get(ao, "playerCoordinate").clone()));
                }
            }
            for k in ["cards", "cardEffects"] {
                if get(bo, k) != get(ao, k) {
                    self.uncovered.push(format!("fieldModel.playerData[{id}].{k}"));
                }
            }
        }
        if bp.iter().any(|(id, _)| !ap.iter().any(|(aid, _)| aid == id)) {
            self.uncovered.push("fieldModel.playerDataArray (player removed)".into());
        }
        Ok(())
    }

    fn game_result(&mut self, b: &Map<String, Value>, a: &Map<String, Value>) -> Result<(), String> {
        for (side, k) in [("home", "teamResultHome"), ("away", "teamResultAway")] {
            let (bt, at) = (obj(get(b, k), k)?, obj(get(a, k), k)?);
            // pettyCashFromTvDiff, treasuryUsedOnInducements and stalling have setters that do not
            // notify in Java: they never travel in a model sync (only in a whole serverGameState).
            self.scalars(k, bt, at, TEAM_RESULT_KEYS, Some(side),
                &["playerResults", "pettyCashFromTvDiff", "treasuryUsedOnInducements", "stalling"]);
            let empty = Vec::new();
            let bp = get(bt, "playerResults").as_array().unwrap_or(&empty);
            let ap = get(at, "playerResults").as_array().unwrap_or(&empty);
            for ae in ap {
                let ao = obj(ae, "playerResult")?;
                let id = get(ao, "playerId").as_str().ok_or("playerResult.playerId")?;
                let blank = Map::new();
                let bo = bp
                    .iter()
                    .filter_map(|e| e.as_object())
                    .find(|o| get(o, "playerId").as_str() == Some(id))
                    .unwrap_or(&blank);
                if bo.is_empty() {
                    self.uncovered.push(format!("{k}.playerResults[{id}] (added)"));
                    continue;
                }
                self.scalars(&format!("{k}.playerResults[{id}]"), bo, ao, PLAYER_RESULT_KEYS, Some(id), &["playerId"]);
            }
        }
        Ok(())
    }
}

/// The model changes that take `before` to `after`, or `Err(paths)` when a changed value has no
/// model change here yet (the caller sends the whole game instead).
pub fn diff(before: &Game, after: &Game) -> Result<ModelChangeList, Vec<String>> {
    let b = before.to_json_value().map_err(|e| vec![format!("before: {e}")])?;
    let a = after.to_json_value().map_err(|e| vec![format!("after: {e}")])?;
    let dialog_set = before.dialog_set_count != after.dialog_set_count
        && !(before.dialog_parameter.is_none() && after.dialog_parameter.is_none());
    let mut d = Differ { list: ModelChangeList::default(), uncovered: Vec::new(), dialog_set };
    d.game(&b, &a).map_err(|e| vec![e])?;
    if d.uncovered.is_empty() {
        Ok(d.list)
    } else {
        Err(d.uncovered)
    }
}

/// Java `ModelChangeProcessor.transform(ModelChange)`: the change as the away coach receives it.
/// Coordinates (and the objects carrying them) mirrored, the dialog parameter transformed,
/// homePlaying / homeFirstOffense negated, the home/away key of team-result, turn-data and
/// inducement-set changes swapped; everything else unchanged.
pub fn transform_change(game: &Game, c: &ModelChange) -> Result<ModelChange, String> {
    use super::model_change_data_type::ModelChangeValue as V;
    use crate::util::java_json as j;
    use ModelChangeId as Id;
    let decoded = || DecodedModelChange::decode(game, c).map(|d| d.value);
    let with_value = |value: Value| ModelChange::with_key(c.change_id, c.key.clone(), value);
    let swapped_key = || {
        let key = if c.get_key() == Some(ModelChange::HOME) { ModelChange::AWAY } else { ModelChange::HOME };
        ModelChange::with_key(c.change_id, Some(key.to_string()), c.value.clone())
    };
    Ok(match c.change_id {
        Id::FieldModelAddBloodSpot => with_value(match decoded()? {
            V::BloodSpot(b) => crate::model::blood_spot::BloodSpot { injury: b.injury, coordinate: b.coordinate.map(|x| x.transform()) }.to_json_value(),
            _ => Value::Null,
        }),
        Id::FieldModelAddDiceDecoration | Id::FieldModelRemoveDiceDecoration => with_value(match decoded()? {
            V::DiceDecoration(d) => d.transform().to_json_value(),
            _ => Value::Null,
        }),
        Id::FieldModelAddFieldMarker | Id::FieldModelRemoveFieldMarker => with_value(match decoded()? {
            V::FieldMarker(m) => m.transform().to_json_value(),
            _ => Value::Null,
        }),
        Id::FieldModelAddMoveSquare | Id::FieldModelRemoveMoveSquare => with_value(match decoded()? {
            V::MoveSquare(m) => m.transform().to_json_value(),
            _ => Value::Null,
        }),
        Id::FieldModelAddPlayerMarker | Id::FieldModelRemovePlayerMarker => with_value(match decoded()? {
            V::PlayerMarker(m) => m.transform().to_json_value(),
            _ => Value::Null,
        }),
        Id::FieldModelAddPushbackSquare | Id::FieldModelRemovePushbackSquare => with_value(match decoded()? {
            V::PushbackSquare(p) => p.transform().to_json_value(),
            _ => Value::Null,
        }),
        Id::FieldModelAddTrackNumber | Id::FieldModelRemoveTrackNumber => with_value(match decoded()? {
            V::TrackNumber(t) => crate::model::track_number::TrackNumber { coordinate: t.coordinate.map(|x| x.transform()), number: t.number }.to_json_value(),
            _ => Value::Null,
        }),
        Id::FieldModelAddTrapDoor | Id::FieldModelRemoveTrapDoor => with_value(match decoded()? {
            V::TrapDoor(t) => {
                let mut o = Map::new();
                o.insert("coordinate".into(), j::field_coordinate_or_null(Some(t.transform())));
                Value::Object(o)
            }
            _ => return Err(format!("{}: null trap door (NPE)", c.change_id.get_name())),
        }),
        Id::FieldModelSetBallCoordinate | Id::FieldModelSetBombCoordinate | Id::FieldModelSetPlayerCoordinate | Id::GameSetPassCoordinate => {
            with_value(match decoded()? {
                V::FieldCoordinate(x) => j::field_coordinate_or_null(Some(x.transform())),
                _ => Value::Null,
            })
        }
        Id::FieldModelSetRangeRuler => with_value(match decoded()? {
            V::RangeRuler(r) => r.transform().to_json_value(),
            _ => Value::Null,
        }),
        Id::GameSetDialogParameter => with_value(match decoded()? {
            V::DialogParameter(p) => p.transform().to_json_value(game.rules)?,
            _ => Value::Null,
        }),
        Id::GameSetHomeFirstOffense | Id::GameSetHomePlaying => match decoded()? {
            V::Boolean(b) => with_value(Value::Bool(!b)),
            _ => return Err(format!("{}: null Boolean (NPE)", c.change_id.get_name())),
        },
        Id::InducementSetActivateCard
        | Id::InducementSetAddAvailableCard
        | Id::InducementSetAddInducement
        | Id::InducementSetAddPrayer
        | Id::InducementSetRemovePrayer
        | Id::InducementSetDeactivateCard
        | Id::InducementSetRemoveAvailableCard
        | Id::InducementSetRemoveInducement
        | Id::TeamResultSetConceded
        | Id::TeamResultSetDedicatedFansModifier
        | Id::TeamResultSetFame
        | Id::TeamResultSetBadlyHurtSuffered
        | Id::TeamResultSetFanFactorModifier
        | Id::TeamResultSetPenaltyScore
        | Id::TeamResultSetPettyCashTransferred
        | Id::TeamResultSetPettyCashUsed
        | Id::TeamResultSetRaisedDead
        | Id::TeamResultSetRipSuffered
        | Id::TeamResultSetScore
        | Id::TeamResultSetSeriousInjurySuffered
        | Id::TeamResultSetSpectators
        | Id::TeamResultSetSpirallingExpenses
        | Id::TeamResultSetTeamValue
        | Id::TeamResultSetFanFactor
        | Id::TeamResultSetWinnings
        | Id::TurnDataSetApothecaries
        | Id::TurnDataSetPlagueDoctors
        | Id::TurnDataSetWanderingApothecaries
        | Id::TurnDataSetBlitzUsed
        | Id::TurnDataSetBombUsed
        | Id::TurnDataSetFirstTurnAfterKickoff
        | Id::TurnDataSetFoulUsed
        | Id::TurnDataSetHandOverUsed
        | Id::TurnDataSetLeaderState
        | Id::TurnDataSetPassUsed
        | Id::TurnDataSetTtmUsed
        | Id::TurnDataSetKtmUsed
        | Id::TurnDataSetSecureTheBallUsed
        | Id::TurnDataSetPuntUsed
        | Id::TurnDataSetReRolls
        | Id::TurnDataSetReRollsBrilliantCoachingOneDrive
        | Id::TurnDataSetReRollsPumpUpTheCrowdOneDrive
        | Id::TurnDataSetReRollsShowStarOneDrive
        | Id::TurnDataSetReRollsSingleUse
        | Id::TurnDataSetReRollUsed
        | Id::TurnDataSetTurnNr
        | Id::TurnDataSetTurnStarted
        | Id::TurnDataSetCoachBanned => swapped_key(),
        _ => c.clone(),
    })
}

/// Java `ModelChangeList.transform()`.
pub fn transform_list(game: &Game, list: &ModelChangeList) -> Result<ModelChangeList, String> {
    let mut out = ModelChangeList::default();
    for c in list.iter() {
        out.add(transform_change(game, c)?);
    }
    Ok(out)
}

/// Java `ModelChangeList.toJsonValue()`: `{"modelChangeArray": [{modelChangeId, modelChangeKey,
/// modelChangeValue}, ...]}` (the key written as null when absent).
pub fn list_to_json_value(list: &ModelChangeList) -> Value {
    let changes = list
        .iter()
        .map(|c| {
            let mut o = Map::new();
            o.insert("modelChangeId".into(), Value::from(c.change_id.get_name()));
            o.insert("modelChangeKey".into(), c.key.clone().map_or(Value::Null, Value::from));
            o.insert("modelChangeValue".into(), c.value.clone());
            Value::Object(o)
        })
        .collect();
    let mut o = Map::new();
    o.insert("modelChangeArray".into(), Value::Array(changes));
    Value::Object(o)
}

/// Every array sorted by its elements' text: the element sets (`HashSet`s on the Java side, a
/// `HashMap` for the Rust move squares) carry no order the client depends on.
fn canonical(v: &Value) -> Value {
    match v {
        Value::Array(items) => {
            let mut items: Vec<Value> = items.iter().map(canonical).collect();
            items.sort_by_key(|x| x.to_string());
            Value::Array(items)
        }
        Value::Object(o) => Value::Object(o.iter().map(|(k, v)| (k.clone(), canonical(v))).collect()),
        other => other.clone(),
    }
}

/// Applies `list` to a copy of `before` with the Java `ModelChangeProcessor` and checks the result
/// against `after` (game and turn time excluded: they travel in the sync command).
pub fn self_check(before: &Game, after: &Game, list: &ModelChangeList) -> Result<(), String> {
    self_check_with(before, after, list, |_| {})
}

/// `self_check` with `normalize` applied to both the replayed and the expected game first.
pub fn self_check_with(before: &Game, after: &Game, list: &ModelChangeList, normalize: impl Fn(&mut Game)) -> Result<(), String> {
    let mut expected = after.clone();
    normalize(&mut expected);
    let after = &expected;
    let mut game = before.clone();
    let processor = ModelChangeProcessor::new();
    for c in list.iter() {
        let decoded = DecodedModelChange::decode(&game, c)?;
        processor.apply(&mut game, &decoded)?;
    }
    game.turn_time = after.turn_time;
    game.game_time = after.game_time;
    for (g, a) in [(&mut game.game_result.home, &after.game_result.home), (&mut game.game_result.away, &after.game_result.away)] {
        g.petty_cash_from_tv_diff = a.petty_cash_from_tv_diff;
        g.treasury_spent_on_inducements = a.treasury_spent_on_inducements;
        g.stalled = a.stalled;
    }
    normalize(&mut game);
    let got = canonical(&game.to_json_value()?);
    let want = canonical(&after.to_json_value()?);
    if got == want {
        return Ok(());
    }
    let (g, w) = (got.as_object().cloned().unwrap_or_default(), want.as_object().cloned().unwrap_or_default());
    let mut differing: Vec<String> = Vec::new();
    for k in w.keys().filter(|k| g.get(*k) != w.get(*k)) {
        match (g.get(k).and_then(Value::as_object), w.get(k).and_then(Value::as_object)) {
            (Some(go), Some(wo)) => {
                for sk in wo.keys().filter(|sk| go.get(*sk) != wo.get(*sk)) {
                    let show = |v: Option<&Value>| v.map(|v| v.to_string()).unwrap_or_default().chars().take(80).collect::<String>();
                    differing.push(format!("{k}.{sk} got {} want {}", show(go.get(sk)), show(wo.get(sk))));
                }
            }
            _ => differing.push(k.clone()),
        }
    }
    Err(format!("model change list does not reproduce the game; differing keys: {differing:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::{PlayerState, Rules, TurnMode};
    use crate::model::player::Player;
    use crate::model::team::Team;
    use crate::types::FieldCoordinate;

    fn game() -> Game {
        let mut home = Team::default();
        home.id = "h".into();
        let mut p = Player::default();
        p.id = "h1".into();
        p.nr = 1;
        home.players.push(p);
        let mut away = Team::default();
        away.id = "a".into();
        let mut g = Game::new(home, away, Rules::Bb2025);
        g.options.init_from(&serde_json::json!({"gameOptionArray": [
            {"gameOptionId": "rulesVersion", "gameOptionValue": "BB2025"}]})).unwrap();
        g
    }

    #[test]
    fn unchanged_game_has_no_changes() {
        let g = game();
        assert!(diff(&g, &g).unwrap().is_empty());
    }

    #[test]
    fn scalars_turn_data_field_and_results_round_trip() {
        let before = game();
        let mut after = before.clone();
        after.turn_mode = TurnMode::Setup;
        after.home_playing = false;
        after.defender_id = Some("h1".into());
        after.turn_data_away.turn_nr = 2;
        after.turn_data_home.reroll_used = true;
        after.field_model.ball_coordinate = Some(FieldCoordinate::new(10, 7));
        after.field_model.set_player_state("h1", PlayerState::new(1));
        after.field_model.set_player_coordinate("h1", FieldCoordinate::new(12, 7));
        after.game_result.away.score = 1;
        after.acting_player.player_id = Some("h1".into());
        after.acting_player.current_move = 3;
        let list = diff(&before, &after).unwrap();
        assert!(list.iter().any(|c| c.change_id == ModelChangeId::GameSetTurnMode && c.value == TurnMode::Setup.name()));
        assert!(list.iter().any(|c| c.change_id == ModelChangeId::TurnDataSetTurnNr && c.get_key() == Some("away")));
        self_check(&before, &after, &list).unwrap();
    }

    /// The away coach's stream: the transformed list applied to the transformed `before` gives
    /// the transformed `after` (Java's `transform` on both the game and every change).
    #[test]
    fn transformed_list_reproduces_the_transformed_game() {
        let before = game();
        let mut after = before.clone();
        after.home_playing = false;
        after.turn_data_home.turn_nr = 4;
        after.game_result.home.score = 2;
        after.field_model.ball_coordinate = Some(FieldCoordinate::new(3, 7));
        after.field_model.set_player_state("h1", PlayerState::new(1));
        after.field_model.set_player_coordinate("h1", FieldCoordinate::new(20, 2));
        after.pass_coordinate = Some(FieldCoordinate::new(9, 9));
        let list = diff(&before, &after).unwrap();
        let away = transform_list(&before, &list).unwrap();
        self_check(&before.transform(), &after.transform(), &away).unwrap();
    }

    #[test]
    fn uncovered_change_is_reported() {
        let before = game();
        let mut after = before.clone();
        after.team_home.name = "renamed".into();
        assert_eq!(diff(&before, &after).unwrap_err(), vec!["teamHome.teamName".to_string()]);
    }

    /// Java RosterPlayer.markUsed -> playerMarkSkillUsed keyed by the player.
    #[test]
    fn used_skill_becomes_player_mark_skill_used() {
        let mut before = game();
        before.team_home.players[0].extra_skills.push(crate::model::skill_def::SkillWithValue::new(crate::enums::SkillId::Block));
        let mut after = before.clone();
        after.team_home.players[0].used_skills.insert(crate::enums::SkillId::Block);
        let list = diff(&before, &after).unwrap();
        assert!(list.iter().any(|c| c.change_id == ModelChangeId::PlayerMarkSkillUsed && c.get_key() == Some("h1")));
        self_check(&before, &after, &list).unwrap();
    }
}
