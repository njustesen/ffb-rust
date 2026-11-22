//! 1:1 translation of com.fumbbl.ffb.model.change.ModelChangeProcessor (`apply`), plus the
//! client-side model mutators it calls, ported here in their JAVA form (BACKLOG §O).
//!
//! The engine's own `FieldModel`/`ActingPlayer` methods carry server-side behaviour (the bb2025
//! `ChompRemovalObserver`, Rust-only bridge flags) that a replaying client does not have, so the
//! processor does NOT call them: each `java_*` helper below is the Java method it names, applied to
//! the fields the Java-JSON writers (`to_json_value`) read. A client-side Java `Game` has no
//! model-change observers, so `notifyObservers` is a no-op throughout.

use super::model_change::ModelChange;
use super::model_change_data_type::ModelChangeValue;
use crate::enums::{ModelChangeId, PlayerState, SkillId, TurnMode};
use crate::model::game::Game;
use crate::model::game_result::{PlayerResult, TeamResult};
use crate::model::inducement_set::InducementSet;
use crate::model::player::Player;
use crate::model::turn_data::TurnData;
use crate::types::FieldCoordinate;

/// A model change whose value has been decoded (`ModelChange.initFrom` decodes the value with
/// the change id's data type; see `ModelChangeDataType::from_json_value`).
#[derive(Debug, Clone)]
pub struct DecodedModelChange {
    pub change_id: ModelChangeId,
    pub key: Option<String>,
    pub value: ModelChangeValue,
}

impl DecodedModelChange {
    /// Java: `ModelChange.initFrom` -- the value decoded by `getDataType().fromJsonValue` with the
    /// game as the factory source.
    pub fn decode(game: &Game, change: &ModelChange) -> Result<Self, String> {
        let value = change
            .change_id
            .data_type()
            .from_json_value(game.rules, &game.options, &change.value)
            .map_err(|e| format!("{}: {e}", change.change_id.get_name()))?;
        Ok(Self { change_id: change.change_id, key: change.key.clone(), value })
    }
}

/// Java: `ModelChangeProcessor`.
#[derive(Debug, Default, Clone, Copy)]
pub struct ModelChangeProcessor;

fn unbox_bool(change: &DecodedModelChange) -> Result<bool, String> {
    match &change.value {
        ModelChangeValue::Boolean(b) => Ok(*b),
        ModelChangeValue::Null => Err(format!("{}: null Boolean unboxed (NPE)", change.change_id.get_name())),
        other => Err(format!("{}: {other:?} is not a Boolean (ClassCastException)", change.change_id.get_name())),
    }
}

fn unbox_int(change: &DecodedModelChange) -> Result<i32, String> {
    match &change.value {
        ModelChangeValue::Integer(i) => Ok(*i),
        ModelChangeValue::Null => Err(format!("{}: null Integer unboxed (NPE)", change.change_id.get_name())),
        other => Err(format!("{}: {other:?} is not an Integer (ClassCastException)", change.change_id.get_name())),
    }
}

fn cast_err(change: &DecodedModelChange) -> String {
    format!("{}: unexpected value {:?} (ClassCastException)", change.change_id.get_name(), change.value)
}

fn opt_string(change: &DecodedModelChange) -> Result<Option<String>, String> {
    match &change.value {
        ModelChangeValue::Null => Ok(None),
        ModelChangeValue::String(s) => Ok(Some(s.clone())),
        _ => Err(cast_err(change)),
    }
}

fn opt_coordinate(change: &DecodedModelChange) -> Result<Option<FieldCoordinate>, String> {
    match &change.value {
        ModelChangeValue::Null => Ok(None),
        ModelChangeValue::FieldCoordinate(c) => Ok(Some(*c)),
        _ => Err(cast_err(change)),
    }
}

fn opt_player_state(change: &DecodedModelChange) -> Result<Option<PlayerState>, String> {
    match &change.value {
        ModelChangeValue::Null => Ok(None),
        ModelChangeValue::PlayerState(s) => Ok(Some(*s)),
        _ => Err(cast_err(change)),
    }
}

fn opt_skill(change: &DecodedModelChange) -> Result<Option<SkillId>, String> {
    match &change.value {
        ModelChangeValue::Null => Ok(None),
        ModelChangeValue::Skill(s) => Ok(Some(*s)),
        _ => Err(cast_err(change)),
    }
}

fn opt_player_action(change: &DecodedModelChange) -> Result<Option<crate::enums::PlayerAction>, String> {
    match &change.value {
        ModelChangeValue::Null => Ok(None),
        ModelChangeValue::PlayerAction(a) => Ok(Some(*a)),
        _ => Err(cast_err(change)),
    }
}

fn opt_date(change: &DecodedModelChange) -> Result<Option<String>, String> {
    match &change.value {
        ModelChangeValue::Null => Ok(None),
        ModelChangeValue::Date(d) => Ok(Some(d.clone())),
        _ => Err(cast_err(change)),
    }
}

fn opt_turn_mode(change: &DecodedModelChange) -> Result<Option<TurnMode>, String> {
    match &change.value {
        ModelChangeValue::Null => Ok(None),
        ModelChangeValue::TurnMode(m) => Ok(Some(*m)),
        _ => Err(cast_err(change)),
    }
}

/// Java: `Game.getPlayerById(id)` -- home team first, then away; null id -> null.
fn player_side(game: &Game, id: Option<&str>) -> Option<bool> {
    let id = id?;
    if game.team_home.has_player(id) {
        Some(true)
    } else if game.team_away.has_player(id) {
        Some(false)
    } else {
        None
    }
}

fn player_mut<'a>(game: &'a mut Game, id: &str) -> Option<&'a mut Player> {
    match player_side(game, Some(id))? {
        true => game.team_home.player_mut(id),
        false => game.team_away.player_mut(id),
    }
}

/// `getPlayerById(key)` where Java goes on to dereference the player (NPE when absent).
fn require_player<'a>(game: &'a mut Game, change: &DecodedModelChange) -> Result<&'a mut Player, String> {
    let name = change.change_id.get_name();
    let key = change.key.clone();
    let id = key.as_deref().ok_or_else(|| format!("{name}: null player key (NPE)"))?;
    player_mut(game, id).ok_or_else(|| format!("{name}: no player {id} (NPE)"))
}

/// Java: `GameResult.getPlayerResult(game.getPlayerById(key))` -- the home result when the home
/// team holds the player, else the away team's; a player neither holds yields null (NPE on use).
fn player_result_mut<'a>(game: &'a mut Game, change: &DecodedModelChange) -> Result<&'a mut PlayerResult, String> {
    let name = change.change_id.get_name();
    let id = change.key.clone().ok_or_else(|| format!("{name}: null player key (NPE)"))?;
    let side = player_side(game, Some(&id)).ok_or_else(|| format!("{name}: no player {id} (NPE)"))?;
    Ok(game.game_result.team_result_mut(side).player_result_mut(&id))
}

/// Java: `ModelChange.HOME.equals(change.getKey())`.
fn is_home_data(change: &DecodedModelChange) -> bool {
    change.key.as_deref() == Some(ModelChange::HOME)
}

fn turn_data_mut<'a>(game: &'a mut Game, change: &DecodedModelChange) -> &'a mut TurnData {
    if is_home_data(change) { &mut game.turn_data_home } else { &mut game.turn_data_away }
}

fn team_result_mut<'a>(game: &'a mut Game, change: &DecodedModelChange) -> &'a mut TeamResult {
    game.game_result.team_result_mut(is_home_data(change))
}

fn inducement_set_mut<'a>(game: &'a mut Game, change: &DecodedModelChange) -> &'a mut InducementSet {
    &mut turn_data_mut(game, change).inducement_set
}

// ── ActingPlayer (Java setters) ─────────────────────────────────────────────

/// Java: `ActingPlayer.setPlayerId`. The strength reset (`setStrength(getStrengthWithModifiers())`)
/// is not part of `toJsonValue`, and the next change in every corpus sync sets it anyway.
fn java_set_acting_player_id(game: &mut Game, player_id: Option<String>) {
    let ap = &mut game.acting_player;
    if ap.player_id == player_id {
        return;
    }
    ap.old_player_state = None;
    ap.player_id = player_id;
    ap.used_skills.clear();
    ap.current_move = 0;
    ap.goes_for_it = false;
    ap.dodging = false;
    ap.has_blocked = false;
    ap.has_fouled = false;
    ap.has_passed = false;
    ap.has_moved = false;
    ap.has_fed = false;
    ap.jumping = false;
    ap.player_action = None;
    ap.standing_up = false;
    ap.suffering_blood_lust = false;
    ap.suffering_animosity = false;
    ap.fumblerooskie_pending = false;
    ap.jumps_without_modifiers = false;
    ap.held_in_place = false;
    ap.must_complete_action = false;
    ap.fell_from_rush = false;
    ap.has_triggered_effect = false;
    ap.forgone = false;
    // Rust-only per-activation fields, reset like the engine's set_player does.
    ap.defender_id = None;
    ap.has_acted = false;
    ap.took_square = false;
    ap.blood_lust_discards_move_stack = false;
    let strength = match game.acting_player.player_id.clone() {
        Some(id) => player_mut(game, &id).map_or(0, |p| p.strength_with_modifiers()),
        None => 0,
    };
    game.acting_player.strength = strength;
    game.acting_player.skills_granted_by.clear();
}

/// `skill.getSkillUsageType().isTrackOutsideActivation()` for the game's ruleset class of the
/// skill (Java-exact table; a skill the ruleset has no class for cannot be a Java `Skill` here).
fn java_tracks_outside_activation(game: &Game, skill: SkillId) -> Result<bool, String> {
    crate::factory::skill_factory::SkillFactory::usage_type_in(game.rules, skill)
        .map(|u| u.track_outside_activation())
        .ok_or_else(|| format!("{skill:?} is not a {:?} skill", game.rules))
}

/// Java: `ActingPlayer.markSkillUsed(Skill)`.
fn java_acting_player_mark_skill_used(game: &mut Game, skill: Option<SkillId>) -> Result<(), String> {
    let Some(skill) = skill else { return Ok(()) };
    if game.acting_player.used_skills.contains(&skill) {
        return Ok(());
    }
    game.acting_player.used_skills.insert(skill);
    if java_tracks_outside_activation(game, skill)? {
        let id = game.acting_player.player_id.clone();
        let player = id
            .as_deref()
            .and_then(|id| player_mut(game, id))
            .ok_or("actingPlayerMarkSkillUsed: getPlayer() is null (NPE)")?;
        if !player.used_skills.contains(&skill) {
            player.used_skills.insert(skill);
        }
    }
    Ok(())
}

/// Java: `ActingPlayer.markSkillUnused(Skill)` (`isSkillUsed` = own set or the player's).
fn java_acting_player_mark_skill_unused(game: &mut Game, skill: Option<SkillId>) -> Result<(), String> {
    let Some(skill) = skill else { return Ok(()) };
    let id = game.acting_player.player_id.clone();
    if !game.acting_player.used_skills.contains(&skill) {
        // isSkillUsed: `fUsedSkills.contains(skill) || getPlayer().isUsed(skill)`.
        let player = id
            .as_deref()
            .and_then(|id| player_mut(game, id))
            .ok_or("actingPlayerMarkSkillUnused: getPlayer() is null (NPE)")?;
        if !player.used_skills.contains(&skill) {
            return Ok(());
        }
    }
    game.acting_player.used_skills.remove(&skill);
    if java_tracks_outside_activation(game, skill)? {
        let player = id
            .as_deref()
            .and_then(|id| player_mut(game, id))
            .ok_or("actingPlayerMarkSkillUnused: getPlayer() is null (NPE)")?;
        player.used_skills.remove(&skill);
    }
    Ok(())
}

// ── FieldModel (Java methods) ───────────────────────────────────────────────

/// Java: `FieldModel.setPlayerCoordinate(player, coordinate)` (no-op for a null player/coordinate).
fn java_set_player_coordinate(game: &mut Game, player_id: Option<&str>, coordinate: Option<FieldCoordinate>) {
    let (Some(_), Some(coordinate)) = (player_side(game, player_id), coordinate) else { return };
    let id = player_id.unwrap_or_default();
    let fm = &mut game.field_model;
    if fm.player_coordinates.get(id) != Some(&coordinate) {
        fm.player_coordinates.insert(id.to_string(), coordinate);
        fm.arrival_counter += 1;
        fm.coordinate_arrival.insert(id.to_string(), fm.arrival_counter);
    }
}

/// Java: `FieldModel.setPlayerState(player, state, false)`: stored when there is no state yet,
/// or the new state is non-null with a different id.
fn java_set_player_state(game: &mut Game, player_id: Option<&str>, state: Option<PlayerState>) {
    if player_side(game, player_id).is_none() {
        return;
    }
    let id = player_id.unwrap_or_default();
    let fm = &mut game.field_model;
    let old = fm.player_states.get(id).copied();
    match (old, state) {
        (None, None) => {}
        (None, Some(s)) => {
            fm.player_states.insert(id.to_string(), s);
        }
        (Some(o), Some(s)) if o.0 != s.0 => {
            fm.player_states.insert(id.to_string(), s);
        }
        _ => {}
    }
}

/// Java: `FieldModel.getPlayerState(player)` -- absent reads as UNKNOWN (0).
fn java_get_player_state(game: &Game, player_id: &str) -> PlayerState {
    game.field_model.player_states.get(player_id).copied().unwrap_or(PlayerState(0))
}

/// Java: `FieldModel.remove(Player)` -- the coordinate goes, the state stays.
fn java_remove_player(game: &mut Game, player_id: Option<&str>) {
    if player_side(game, player_id).is_none() {
        return;
    }
    let id = player_id.unwrap_or_default();
    game.field_model.player_coordinates.remove(id);
    game.field_model.coordinate_arrival.remove(id);
}

/// Java: `FieldModel.addChomp(chomper, chompee)`.
fn java_add_chomp(game: &mut Game, chomper: Option<&str>, chompee: Option<&str>) -> Result<(), String> {
    let (Some(_), Some(_)) = (player_side(game, chomper), player_side(game, chompee)) else {
        return Err("fieldModelAddChomp: null chomper/chompee (NPE)".into());
    };
    let (chomper, chompee) = (chomper.unwrap_or_default(), chompee.unwrap_or_default());
    let chompees = game.field_model.chomped.entry(chomper.to_string()).or_default();
    if !chompees.iter().any(|c| c == chompee) {
        chompees.push(chompee.to_string());
        let state = java_get_player_state(game, chompee).change_chomped(true);
        java_set_player_state(game, Some(chompee), Some(state));
    }
    Ok(())
}

/// Java: `FieldModel.removeChomp(chomper, chompee)`.
fn java_remove_chomp(game: &mut Game, chomper: Option<&str>, chompee: Option<&str>) -> Result<(), String> {
    let (Some(_), Some(_)) = (player_side(game, chomper), player_side(game, chompee)) else {
        return Err("fieldModelRemoveChomp: null chomper/chompee (NPE)".into());
    };
    let (chomper, chompee) = (chomper.unwrap_or_default(), chompee.unwrap_or_default());
    let removed = match game.field_model.chomped.get_mut(chomper) {
        Some(list) => match list.iter().position(|c| c == chompee) {
            Some(pos) => {
                list.remove(pos);
                true
            }
            None => false,
        },
        None => false,
    };
    if removed && !game.field_model.chomped.values().any(|l| l.iter().any(|c| c == chompee)) {
        let state = java_get_player_state(game, chompee).change_chomped(false);
        java_set_player_state(game, Some(chompee), Some(state));
    }
    Ok(())
}

/// Java: `Player.removeEnhancements(String)` = remove the source from the temporary modifier,
/// property and skill maps (the engine caches follow).
fn java_remove_enhancements(player: &mut Player, source: &str) {
    player.temporary_modifiers_map.retain(|(s, _)| s != source);
    player.temporary_properties_map.retain(|(s, _)| s != source);
    player.temporary_skills_map.retain(|(s, _)| s != source);
    player.remove_enhancements(source);
}

// ── Game (Java setters with side effects) ───────────────────────────────────

/// Java: `Game.setLastTurnMode`.
fn java_set_last_turn_mode(game: &mut Game, mode: Option<TurnMode>) {
    game.last_turn_mode = mode;
}

/// Java: `Game.setTurnMode`: a change moves the old (non-null) mode into `lastTurnMode`.
fn java_set_turn_mode(game: &mut Game, mode: Option<TurnMode>) {
    let old = if game.turn_mode_null { None } else { Some(game.turn_mode) };
    if mode == old {
        return;
    }
    match mode {
        Some(m) => {
            game.turn_mode = m;
            game.turn_mode_null = false;
        }
        None => game.turn_mode_null = true,
    }
    if let Some(old) = old {
        java_set_last_turn_mode(game, Some(old));
    }
}

impl ModelChangeProcessor {
    pub fn new() -> Self {
        Self
    }

    /// Java: `ModelChangeProcessor.apply(game, change)`. `Ok(false)` where Java returns false (an
    /// id the switch does not handle: `sketchUpdate`); `Err` where Java throws, and for the
    /// changes whose model side is not ported yet (named in the message) -- never silent.
    pub fn apply(&self, game: &mut Game, change: &DecodedModelChange) -> Result<bool, String> {
        use ModelChangeId as Id;
        use ModelChangeValue as V;
        let key = change.key.clone();
        let key = key.as_deref();
        match change.change_id {
            Id::ActingPlayerMarkSkillUsed => java_acting_player_mark_skill_used(game, opt_skill(change)?)?,
            Id::ActingPlayerMarkSkillUnused => java_acting_player_mark_skill_unused(game, opt_skill(change)?)?,
            Id::ActingPlayerSetCurrentMove => game.acting_player.current_move = unbox_int(change)?,
            Id::ActingPlayerSetDodging => game.acting_player.dodging = unbox_bool(change)?,
            Id::ActingPlayerSetGoingForIt => game.acting_player.goes_for_it = unbox_bool(change)?,
            Id::ActingPlayerSetHasBlocked => game.acting_player.has_blocked = unbox_bool(change)?,
            Id::ActingPlayerSetHasFed => game.acting_player.has_fed = unbox_bool(change)?,
            Id::ActingPlayerSetHasFouled => game.acting_player.has_fouled = unbox_bool(change)?,
            // hasJumped is not part of ActingPlayer.toJsonValue (nor of the Rust struct).
            Id::ActingPlayerSetHasJumped => {
                unbox_bool(change)?;
            }
            Id::ActingPlayerSetHasTriggeredEffect => game.acting_player.has_triggered_effect = unbox_bool(change)?,
            Id::ActingPlayerSetHasMoved => game.acting_player.has_moved = unbox_bool(change)?,
            Id::ActingPlayerSetHasPassed => game.acting_player.has_passed = unbox_bool(change)?,
            // setJumping / setJumpsWithoutModifiers carry their Java coupling.
            Id::ActingPlayerSetJumping => game.acting_player.set_jumping(unbox_bool(change)?),
            Id::ActingPlayerSetOldPlayerState => {
                let state = opt_player_state(change)?;
                let ap = &mut game.acting_player;
                if !(ap.old_player_state.is_some() && state.is_some()) {
                    ap.old_player_state = state;
                }
            }
            Id::ActingPlayerSetPlayerAction => game.acting_player.player_action = opt_player_action(change)?,
            Id::ActingPlayerSetPlayerId => java_set_acting_player_id(game, opt_string(change)?),
            Id::ActingPlayerSetStandingUp => game.acting_player.standing_up = unbox_bool(change)?,
            Id::ActingPlayerSetStrength => game.acting_player.strength = unbox_int(change)?,
            Id::ActingPlayerSetSufferingAnimosity => game.acting_player.suffering_animosity = unbox_bool(change)?,
            Id::ActingPlayerSetSufferingBloodLust => game.acting_player.suffering_blood_lust = unbox_bool(change)?,
            Id::ActingPlayerSetJumpsWithoutModifiers => game.acting_player.set_jumps_without_modifiers(unbox_bool(change)?),
            Id::ActingPlayerSetHeldInPlace => game.acting_player.held_in_place = unbox_bool(change)?,
            Id::ActingPlayerSetMustCompleteAction => game.acting_player.must_complete_action = unbox_bool(change)?,
            Id::ActingPlayerSetFellFromRush => game.acting_player.fell_from_rush = unbox_bool(change)?,

            Id::FieldModelAddBloodSpot => match &change.value {
                V::Null => {}
                V::BloodSpot(b) => game.field_model.bloodspots.push(b.clone()),
                _ => return Err(cast_err(change)),
            },
            // HashSet.add keeps the FIRST equal element (equality = coordinate); hashCode of a
            // null coordinate NPEs.
            Id::FieldModelAddDiceDecoration => match &change.value {
                V::Null => {}
                V::DiceDecoration(d) => {
                    if d.coordinate.is_none() {
                        return Err("fieldModelAddDiceDecoration: null coordinate (NPE)".into());
                    }
                    if !game.field_model.dice_decorations.iter().any(|x| x.coordinate == d.coordinate) {
                        game.field_model.dice_decorations.push(d.clone());
                    }
                }
                _ => return Err(cast_err(change)),
            },
            // add(FieldMarker): remove the equal one, then add (= replace).
            Id::FieldModelAddFieldMarker => match &change.value {
                V::Null => {}
                V::FieldMarker(m) => {
                    game.field_model.field_markers.retain(|x| x.coordinate != m.coordinate);
                    game.field_model.field_markers.push(m.clone());
                }
                _ => return Err(cast_err(change)),
            },
            Id::FieldModelAddMoveSquare => match &change.value {
                V::Null => {}
                V::MoveSquare(m) => {
                    game.field_model.move_squares.entry(m.coordinate).or_insert(*m);
                }
                _ => return Err(cast_err(change)),
            },
            Id::FieldModelAddPlayerMarker => match &change.value {
                V::Null => {}
                V::PlayerMarker(m) => {
                    game.field_model.player_markers.retain(|x| x.player_id != m.player_id);
                    game.field_model.player_markers.push(m.clone());
                }
                _ => return Err(cast_err(change)),
            },
            Id::FieldModelAddPushbackSquare => match &change.value {
                V::Null => {}
                V::PushbackSquare(p) => {
                    if !game.field_model.pushback_squares.iter().any(|x| x.coordinate == p.coordinate) {
                        game.field_model.pushback_squares.push(p.clone());
                    }
                }
                _ => return Err(cast_err(change)),
            },
            Id::FieldModelAddTrackNumber => match &change.value {
                V::Null => {}
                V::TrackNumber(t) => {
                    if t.coordinate.is_none() {
                        return Err("fieldModelAddTrackNumber: null coordinate (NPE)".into());
                    }
                    if !game.field_model.track_numbers.iter().any(|x| x.coordinate == t.coordinate) {
                        game.field_model.track_numbers.push(t.clone());
                    }
                }
                _ => return Err(cast_err(change)),
            },
            // trapDoors is a List: add appends (a null trap door would be added as null).
            Id::FieldModelAddTrapDoor => match &change.value {
                V::TrapDoor(c) => game.field_model.trap_doors.push(*c),
                _ => return Err(cast_err(change)),
            },
            Id::FieldModelRemoveDiceDecoration => match &change.value {
                V::DiceDecoration(d) => {
                    if d.coordinate.is_none() {
                        return Err("fieldModelRemoveDiceDecoration: null coordinate (NPE)".into());
                    }
                    game.field_model.dice_decorations.retain(|x| x.coordinate != d.coordinate);
                }
                V::Null => {}
                _ => return Err(cast_err(change)),
            },
            Id::FieldModelRemoveFieldMarker => match &change.value {
                V::FieldMarker(m) => game.field_model.field_markers.retain(|x| x.coordinate != m.coordinate),
                V::Null => {}
                _ => return Err(cast_err(change)),
            },
            Id::FieldModelRemoveMoveSquare => match &change.value {
                V::MoveSquare(m) => {
                    game.field_model.move_squares.remove(&m.coordinate);
                }
                V::Null => {}
                _ => return Err(cast_err(change)),
            },
            Id::FieldModelRemovePlayer => java_remove_player(game, key),
            Id::FieldModelRemovePlayerMarker => match &change.value {
                V::PlayerMarker(m) => game.field_model.player_markers.retain(|x| x.player_id != m.player_id),
                V::Null => {}
                _ => return Err(cast_err(change)),
            },
            Id::FieldModelRemovePushbackSquare => match &change.value {
                V::PushbackSquare(p) => game.field_model.pushback_squares.retain(|x| x.coordinate != p.coordinate),
                V::Null => {}
                _ => return Err(cast_err(change)),
            },
            Id::FieldModelRemoveSkillEnhancements => {
                let name = opt_string(change)?.ok_or("fieldModelRemoveSkillEnhancements: null name")?;
                java_remove_enhancements(require_player(game, change)?, &name);
            }
            Id::FieldModelRemoveTrackNumber => match &change.value {
                V::TrackNumber(t) => game.field_model.track_numbers.retain(|x| x.coordinate != t.coordinate),
                V::Null => {}
                _ => return Err(cast_err(change)),
            },
            // List.remove(Object): the first equal element.
            Id::FieldModelRemoveTrapDoor => match &change.value {
                V::TrapDoor(c) => {
                    if let Some(pos) = game.field_model.trap_doors.iter().position(|x| x == c) {
                        game.field_model.trap_doors.remove(pos);
                    }
                }
                _ => return Err(cast_err(change)),
            },
            Id::FieldModelSetBallCoordinate => game.field_model.ball_coordinate = opt_coordinate(change)?,
            Id::FieldModelSetBallInPlay => game.field_model.ball_in_play = unbox_bool(change)?,
            Id::FieldModelSetBallMoving => game.field_model.ball_moving = unbox_bool(change)?,
            Id::FieldModelSetBlitzState | Id::FieldModelSetTargetSelectionState => match &change.value {
                V::Null => game.field_model.target_selection_state = None,
                V::TargetSelectionState(t) => game.field_model.target_selection_state = Some((**t).clone()),
                _ => return Err(cast_err(change)),
            },
            Id::FieldModelSetBombCoordinate => game.field_model.bomb_coordinate = opt_coordinate(change)?,
            Id::FieldModelSetBombMoving => game.field_model.bomb_moving = unbox_bool(change)?,
            Id::FieldModelSetOutOfBounds => game.field_model.out_of_bounds = unbox_bool(change)?,
            Id::FieldModelSetPlayerCoordinate => java_set_player_coordinate(game, key, opt_coordinate(change)?),
            Id::FieldModelSetPlayerState => java_set_player_state(game, key, opt_player_state(change)?),
            Id::FieldModelSetRangeRuler => match &change.value {
                V::Null => game.field_model.range_ruler = None,
                V::RangeRuler(r) => game.field_model.range_ruler = Some(r.clone()),
                _ => return Err(cast_err(change)),
            },
            Id::FieldModelSetWeather => match &change.value {
                V::Weather(w) => game.field_model.weather = *w,
                V::Null => return Err("fieldModelSetWeather: null weather (the Rust FieldModel cannot hold it)".into()),
                _ => return Err(cast_err(change)),
            },
            Id::FieldModelAddChomp => java_add_chomp(game, key, opt_string(change)?.as_deref())?,
            Id::FieldModelRemoveChomp => java_remove_chomp(game, key, opt_string(change)?.as_deref())?,

            Id::FieldModelAddCard
            | Id::FieldModelAddCardEffect
            | Id::FieldModelKeepDeactivatedCard
            | Id::FieldModelRemoveCard
            | Id::FieldModelRemoveCardEffect
            | Id::FieldModelAddEnhancements
            | Id::FieldModelAddHatred
            | Id::FieldModelAddIntensiveTraining
            | Id::FieldModelAddPrayer
            | Id::FieldModelAddSkillEnhancements
            | Id::FieldModelAddWisdom
            | Id::FieldModelRemovePrayer
            | Id::GameOptionsAddOption
            | Id::InducementSetActivateCard
            | Id::InducementSetAddAvailableCard
            | Id::InducementSetCardChoices
            | Id::InducementSetDeactivateCard
            | Id::InducementSetRemoveAvailableCard => {
                return Err(format!("{}: model change not ported yet", change.change_id.get_name()));
            }

            Id::GameSetConcededLegally => game.conceded_legally = unbox_bool(change)?,
            Id::GameSetConcessionPossible => game.concession_possible = unbox_bool(change)?,
            Id::GameSetDefenderAction => game.defender_action = opt_player_action(change)?,
            Id::GameSetDefenderId => game.defender_id = key.map(str::to_string),
            Id::GameSetDialogParameter => match &change.value {
                V::Null => {
                    game.dialog_parameter = None;
                    game.dialog_id = None;
                }
                V::DialogParameter(p) => {
                    game.dialog_id = Some(p.get_id());
                    game.dialog_parameter = Some((**p).clone());
                }
                _ => return Err(cast_err(change)),
            },
            Id::GameSetFinished => game.finished = opt_date(change)?,
            Id::GameSetHalf => game.half = unbox_int(change)?,
            Id::GameSetHomeFirstOffense => game.home_first_offense = unbox_bool(change)?,
            Id::GameSetHomePlaying => game.home_playing = unbox_bool(change)?,
            // Game.setId also sets both teams' currentGameId (not part of the JSON state).
            Id::GameSetId => match &change.value {
                V::Long(id) => game.id = u64::try_from(*id).map_err(|_| format!("gameSetId: negative id {id}"))?,
                V::Null => return Err("gameSetId: null Long unboxed (NPE)".into()),
                _ => return Err(cast_err(change)),
            },
            Id::GameSetLastDefenderId => game.last_defender_id = key.map(str::to_string),
            Id::GameSetPassCoordinate => game.pass_coordinate = opt_coordinate(change)?,
            Id::GameSetScheduled => game.scheduled = opt_date(change)?,
            Id::GameSetSetupOffense => game.setup_offense = unbox_bool(change)?,
            Id::GameSetStarted => game.started = opt_date(change)?,
            Id::GameSetTesting => game.testing = unbox_bool(change)?,
            Id::GameSetAdminMode => game.admin_mode = unbox_bool(change)?,
            Id::GameSetThrowerId => game.thrower_id = key.map(str::to_string),
            Id::GameSetThrowerAction => game.thrower_action = opt_player_action(change)?,
            Id::GameSetTimeoutEnforced => game.timeout_enforced = unbox_bool(change)?,
            Id::GameSetTimeoutPossible => game.timeout_possible = unbox_bool(change)?,
            Id::GameSetTurnMode => java_set_turn_mode(game, opt_turn_mode(change)?),
            Id::GameSetLastTurnMode => java_set_last_turn_mode(game, opt_turn_mode(change)?),
            Id::GameSetWaitingForOpponent => game.waiting_for_opponent = unbox_bool(change)?,

            Id::InducementSetAddInducement => match &change.value {
                V::Null => {}
                V::Inducement(i) => inducement_set_mut(game, change).add_inducement(i.clone()),
                _ => return Err(cast_err(change)),
            },
            Id::InducementSetAddPrayer => match &change.value {
                V::Null => {}
                V::Prayer(p) => inducement_set_mut(game, change).add_prayer(*p),
                _ => return Err(cast_err(change)),
            },
            Id::InducementSetRemoveInducement => match &change.value {
                V::Null => {}
                V::Inducement(i) => inducement_set_mut(game, change).remove_inducement(&i.type_id),
                _ => return Err(cast_err(change)),
            },
            Id::InducementSetRemovePrayer => match &change.value {
                V::Null => {}
                V::Prayer(p) => inducement_set_mut(game, change).remove_prayer(p),
                _ => return Err(cast_err(change)),
            },

            // RosterPlayer.markUsed / markUnused (null skill -> no-op).
            Id::PlayerMarkSkillUsed => {
                let skill = opt_skill(change)?;
                let player = require_player(game, change)?;
                if let Some(skill) = skill {
                    player.used_skills.insert(skill);
                }
            }
            Id::PlayerMarkSkillUnused => {
                let skill = opt_skill(change)?;
                let player = require_player(game, change)?;
                if let Some(skill) = skill {
                    player.used_skills.remove(&skill);
                }
            }
            Id::PlayerResultSetBlocks => player_result_mut(game, change)?.blocks = unbox_int(change)?,
            Id::PlayerResultSetCasualties => player_result_mut(game, change)?.casualties = unbox_int(change)?,
            Id::PlayerResultSetCatchesWithAdditionalSpp => {
                player_result_mut(game, change)?.catches_with_additional_spp = unbox_int(change)?
            }
            Id::PlayerResultSetCasualtiesWithAdditionalSpp => {
                player_result_mut(game, change)?.casualties_with_additional_spp = unbox_int(change)?
            }
            Id::PlayerResultSetCompletions => player_result_mut(game, change)?.completions = unbox_int(change)?,
            Id::PlayerResultSetCompletionsWithAdditionalSpp => {
                player_result_mut(game, change)?.completions_with_additional_spp = unbox_int(change)?
            }
            Id::PlayerResultSetCurrentSpps => player_result_mut(game, change)?.current_spps = unbox_int(change)?,
            Id::PlayerResultSetDefecting => player_result_mut(game, change)?.defecting = unbox_bool(change)?,
            Id::PlayerResultSetFouls => player_result_mut(game, change)?.fouls = unbox_int(change)?,
            Id::PlayerResultSetHasUsedSecretWeapon => {
                player_result_mut(game, change)?.has_used_secret_weapon = unbox_bool(change)?
            }
            Id::PlayerResultSetInterceptions => player_result_mut(game, change)?.interceptions = unbox_int(change)?,
            Id::PlayerResultSetDeflections => player_result_mut(game, change)?.deflections = unbox_int(change)?,
            Id::PlayerResultSetLandings => player_result_mut(game, change)?.landings = unbox_int(change)?,
            Id::PlayerResultSetPassing => player_result_mut(game, change)?.passing = unbox_int(change)?,
            Id::PlayerResultSetPlayerAwards => player_result_mut(game, change)?.player_awards = unbox_int(change)?,
            Id::PlayerResultSetRushing => player_result_mut(game, change)?.rushing = unbox_int(change)?,
            Id::PlayerResultSetSendToBoxByPlayerId => {
                let v = opt_string(change)?;
                player_result_mut(game, change)?.send_to_box_by_player_id = v;
            }
            Id::PlayerResultSetSendToBoxHalf => player_result_mut(game, change)?.send_to_box_half = unbox_int(change)?,
            Id::PlayerResultSetSendToBoxReason => {
                let v = match &change.value {
                    V::Null => None,
                    V::SendToBoxReason(r) => Some(*r),
                    _ => return Err(cast_err(change)),
                };
                player_result_mut(game, change)?.send_to_box_reason = v;
            }
            Id::PlayerResultSetSendToBoxTurn => player_result_mut(game, change)?.send_to_box_turn = unbox_int(change)?,
            Id::PlayerResultSetSeriousInjury | Id::PlayerResultSetSeriousInjuryDecay => {
                let v = match &change.value {
                    V::Null => None,
                    V::SeriousInjury(s) => Some(*s),
                    _ => return Err(cast_err(change)),
                };
                let result = player_result_mut(game, change)?;
                if change.change_id == Id::PlayerResultSetSeriousInjury {
                    result.serious_injury = v;
                } else {
                    result.serious_injury_decay = v;
                }
            }
            Id::PlayerResultSetTouchdowns => player_result_mut(game, change)?.touchdowns = unbox_int(change)?,
            Id::PlayerResultSetTurnsPlayed => player_result_mut(game, change)?.turns_played = unbox_int(change)?,

            // Not handled by the Java switch: apply returns false.
            Id::SketchUpdate => return Ok(false),

            Id::TargetSelectionCommitted => {
                if let Some(state) = game.field_model.target_selection_state.as_mut() {
                    state.committed = true;
                }
            }

            Id::TeamResultSetConceded => team_result_mut(game, change).conceded = unbox_bool(change)?,
            Id::TeamResultSetDedicatedFansModifier => {
                team_result_mut(game, change).dedicated_fans_modifier = unbox_int(change)?
            }
            Id::TeamResultSetFame => team_result_mut(game, change).fame = unbox_int(change)?,
            Id::TeamResultSetBadlyHurtSuffered => team_result_mut(game, change).badly_hurt_suffered = unbox_int(change)?,
            Id::TeamResultSetFanFactorModifier => team_result_mut(game, change).fan_factor_modifier = unbox_int(change)?,
            Id::TeamResultSetPenaltyScore => team_result_mut(game, change).penalty_score = unbox_int(change)?,
            Id::TeamResultSetPettyCashTransferred => {
                team_result_mut(game, change).petty_cash_transferred = unbox_int(change)?
            }
            Id::TeamResultSetPettyCashUsed => team_result_mut(game, change).petty_cash_used = unbox_int(change)?,
            Id::TeamResultSetRaisedDead => team_result_mut(game, change).raised_dead = unbox_int(change)?,
            Id::TeamResultSetRipSuffered => team_result_mut(game, change).rip_suffered = unbox_int(change)?,
            Id::TeamResultSetScore => team_result_mut(game, change).score = unbox_int(change)?,
            Id::TeamResultSetSeriousInjurySuffered => {
                team_result_mut(game, change).serious_injury_suffered = unbox_int(change)?
            }
            Id::TeamResultSetSpectators => team_result_mut(game, change).spectators = unbox_int(change)?,
            Id::TeamResultSetSpirallingExpenses => team_result_mut(game, change).spiralling_expenses = unbox_int(change)?,
            Id::TeamResultSetTeamValue => team_result_mut(game, change).team_value = unbox_int(change)?,
            Id::TeamResultSetFanFactor => team_result_mut(game, change).fan_factor = unbox_int(change)?,
            Id::TeamResultSetWinnings => team_result_mut(game, change).winnings = unbox_int(change)?,

            Id::TurnDataSetApothecaries => turn_data_mut(game, change).apothecaries = unbox_int(change)?,
            Id::TurnDataSetBlitzUsed => turn_data_mut(game, change).blitz_used = unbox_bool(change)?,
            Id::TurnDataSetBombUsed => turn_data_mut(game, change).bomb_used = unbox_bool(change)?,
            Id::TurnDataSetFirstTurnAfterKickoff => turn_data_mut(game, change).first_turn_after_kickoff = unbox_bool(change)?,
            Id::TurnDataSetFoulUsed => turn_data_mut(game, change).foul_used = unbox_bool(change)?,
            Id::TurnDataSetHandOverUsed => turn_data_mut(game, change).hand_over_used = unbox_bool(change)?,
            Id::TurnDataSetLeaderState => match &change.value {
                V::LeaderState(s) => turn_data_mut(game, change).leader_state = *s,
                V::Null => return Err("turnDataSetLeaderState: null leader state (the Rust TurnData cannot hold it)".into()),
                _ => return Err(cast_err(change)),
            },
            Id::TurnDataSetPassUsed => turn_data_mut(game, change).pass_used = unbox_bool(change)?,
            Id::TurnDataSetTtmUsed => turn_data_mut(game, change).ttm_used = unbox_bool(change)?,
            Id::TurnDataSetKtmUsed => turn_data_mut(game, change).ktm_used = unbox_bool(change)?,
            Id::TurnDataSetSecureTheBallUsed => turn_data_mut(game, change).secure_the_ball_used = unbox_bool(change)?,
            Id::TurnDataSetPuntUsed => turn_data_mut(game, change).punt_used = unbox_bool(change)?,
            Id::TurnDataSetReRolls => turn_data_mut(game, change).rerolls = unbox_int(change)?,
            Id::TurnDataSetReRollsBrilliantCoachingOneDrive => {
                turn_data_mut(game, change).rerolls_brilliant_coaching_one_drive = unbox_int(change)?
            }
            Id::TurnDataSetReRollsPumpUpTheCrowdOneDrive => {
                turn_data_mut(game, change).rerolls_pump_up_the_crowd_one_drive = unbox_int(change)?
            }
            Id::TurnDataSetReRollsShowStarOneDrive => turn_data_mut(game, change).reroll_show_star_one_drive = unbox_int(change)?,
            Id::TurnDataSetReRollsSingleUse => turn_data_mut(game, change).single_use_rerolls = unbox_int(change)?,
            Id::TurnDataSetReRollUsed => turn_data_mut(game, change).reroll_used = unbox_bool(change)?,
            Id::TurnDataSetTurnNr => turn_data_mut(game, change).turn_nr = unbox_int(change)?,
            Id::TurnDataSetTurnStarted => turn_data_mut(game, change).turn_started = unbox_bool(change)?,
            Id::TurnDataSetCoachBanned => turn_data_mut(game, change).coach_banned = unbox_bool(change)?,
            Id::TurnDataSetWanderingApothecaries => turn_data_mut(game, change).wandering_apothecaries = unbox_int(change)?,
            Id::TurnDataSetPlagueDoctors => turn_data_mut(game, change).plague_doctors = unbox_int(change)?,
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::{PlayerAction, Rules};
    use crate::model::team::Team;
    use serde_json::json;

    fn game() -> Game {
        let mut home = Team::new();
        home.players.push(Player { id: "h1".into(), nr: 1, ..Default::default() });
        let mut away = Team::new();
        away.players.push(Player { id: "a1".into(), nr: 1, ..Default::default() });
        Game::new(home, away, Rules::Bb2020)
    }

    fn apply(game: &mut Game, id: &str, key: Option<&str>, value: serde_json::Value) -> Result<bool, String> {
        let change = ModelChange::init_from(&json!({"modelChangeId": id, "modelChangeKey": key, "modelChangeValue": value}))?;
        let decoded = DecodedModelChange::decode(game, &change)?;
        ModelChangeProcessor.apply(game, &decoded)
    }

    #[test]
    fn set_player_id_resets_the_activation_like_java() {
        let mut g = game();
        apply(&mut g, "actingPlayerSetPlayerId", None, json!("h1")).unwrap();
        apply(&mut g, "actingPlayerSetPlayerAction", None, json!("move")).unwrap();
        apply(&mut g, "actingPlayerSetHasMoved", None, json!(true)).unwrap();
        apply(&mut g, "actingPlayerSetOldPlayerState", None, json!(1)).unwrap();
        // a second non-null old state is ignored (setOldPlayerState)
        apply(&mut g, "actingPlayerSetOldPlayerState", None, json!(2)).unwrap();
        assert_eq!(g.acting_player.old_player_state, Some(PlayerState(1)));
        assert_eq!(g.acting_player.player_action, Some(PlayerAction::Move));
        // the same id: no reset
        apply(&mut g, "actingPlayerSetPlayerId", None, json!("h1")).unwrap();
        assert!(g.acting_player.has_moved);
        apply(&mut g, "actingPlayerSetPlayerId", None, json!(null)).unwrap();
        assert!(!g.acting_player.has_moved);
        assert_eq!(g.acting_player.player_action, None);
        assert_eq!(g.acting_player.old_player_state, None);
    }

    #[test]
    fn turn_mode_moves_the_old_mode_to_last_turn_mode() {
        let mut g = game();
        g.turn_mode = TurnMode::Setup;
        apply(&mut g, "gameSetTurnMode", None, json!("kickoff")).unwrap();
        assert_eq!(g.turn_mode, TurnMode::Kickoff);
        assert_eq!(g.last_turn_mode, Some(TurnMode::Setup));
        // an unknown (newer-server) mode is null in Java
        apply(&mut g, "gameSetTurnMode", None, json!("dwarfenWisdom")).unwrap();
        assert!(g.turn_mode_null);
        assert_eq!(g.last_turn_mode, Some(TurnMode::Kickoff));
        assert_eq!(g.to_json_value().unwrap()["turnMode"], json!(null));
        // back from null: lastTurnMode is NOT updated (the old mode was null)
        apply(&mut g, "gameSetTurnMode", None, json!("regular")).unwrap();
        assert!(!g.turn_mode_null);
        assert_eq!(g.last_turn_mode, Some(TurnMode::Kickoff));
    }

    #[test]
    fn field_model_player_state_coordinate_and_remove() {
        let mut g = game();
        apply(&mut g, "fieldModelSetPlayerState", Some("h1"), json!(1)).unwrap();
        apply(&mut g, "fieldModelSetPlayerCoordinate", Some("h1"), json!([5, 6])).unwrap();
        // a null state over a state: ignored
        apply(&mut g, "fieldModelSetPlayerState", Some("h1"), json!(null)).unwrap();
        assert_eq!(g.field_model.player_states.get("h1"), Some(&PlayerState(1)));
        // an unknown player: no-op
        apply(&mut g, "fieldModelSetPlayerState", Some("zz"), json!(1)).unwrap();
        assert!(!g.field_model.player_states.contains_key("zz"));
        // remove(Player): the coordinate goes, the state stays
        apply(&mut g, "fieldModelRemovePlayer", Some("h1"), json!([5, 6])).unwrap();
        assert!(g.field_model.player_coordinates.get("h1").is_none());
        assert_eq!(g.field_model.player_states.get("h1"), Some(&PlayerState(1)));
    }

    #[test]
    fn hash_set_squares_keep_the_first_and_remove_by_coordinate() {
        let mut g = game();
        apply(&mut g, "fieldModelAddMoveSquare", None, json!({"coordinate": [3, 3], "minimumRollDodge": 2, "minimumRollGfi": 0})).unwrap();
        apply(&mut g, "fieldModelAddMoveSquare", None, json!({"coordinate": [3, 3], "minimumRollDodge": 5, "minimumRollGfi": 0})).unwrap();
        assert_eq!(g.field_model.move_squares.len(), 1);
        assert_eq!(g.field_model.move_squares[&FieldCoordinate::new(3, 3)].minimum_roll_dodge, 2);
        apply(&mut g, "fieldModelRemoveMoveSquare", None, json!({"coordinate": [3, 3], "minimumRollDodge": 9, "minimumRollGfi": 0})).unwrap();
        assert!(g.field_model.move_squares.is_empty());
    }

    #[test]
    fn home_away_keys_and_player_results() {
        let mut g = game();
        apply(&mut g, "turnDataSetTurnNr", Some("home"), json!(3)).unwrap();
        apply(&mut g, "turnDataSetTurnNr", Some("away"), json!(4)).unwrap();
        assert_eq!((g.turn_data_home.turn_nr, g.turn_data_away.turn_nr), (3, 4));
        apply(&mut g, "teamResultSetScore", Some("home"), json!(2)).unwrap();
        assert_eq!(g.game_result.home.score, 2);
        apply(&mut g, "playerResultSetBlocks", Some("a1"), json!(5)).unwrap();
        assert_eq!(g.game_result.away.player_results["a1"].blocks, 5);
        assert!(apply(&mut g, "playerResultSetBlocks", Some("zz"), json!(5)).is_err());
        assert!(apply(&mut g, "turnDataSetBlitzUsed", Some("home"), json!(null)).is_err());
        assert!(!apply(&mut g, "sketchUpdate", None, json!(null)).unwrap());
    }
}
