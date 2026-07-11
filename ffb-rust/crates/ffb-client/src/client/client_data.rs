//! 1:1 translation of `com.fumbbl.ffb.client.ClientData`.
//!
//! Java stores `fSelectedPlayer` as a live `Player<?>` reference. This project's convention
//! elsewhere (e.g. `ActingPlayer`) is to reference players by id rather than duplicate mutable
//! player state, so `selected_player` holds a `PlayerId` instead.

use ffb_model::model::player::PlayerId;
use ffb_model::model::{BlockRoll, SpecialEffect, StatusType};
use ffb_model::types::FieldCoordinate;

/// Java: `com.fumbbl.ffb.client.ClientData`.
#[derive(Debug, Clone, Default)]
pub struct ClientData {
    selected_player: Option<PlayerId>,
    drag_end_position: Option<FieldCoordinate>,
    drag_start_position: Option<FieldCoordinate>,
    status_title: Option<String>,
    status_message: Option<String>,
    status_type: Option<StatusType>,
    block_rolls: Vec<BlockRoll>,
    acting_player_updated: bool,
    turn_timer_stopped: bool,
    end_turn_button_hidden: bool,
    spectator_count: i32,
    wizard_spell: Option<SpecialEffect>,
    spectators: Vec<String>,
    coach_controlling_replay: Option<String>,
}

impl ClientData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn selected_player(&self) -> Option<&PlayerId> {
        self.selected_player.as_ref()
    }

    pub fn set_selected_player(&mut self, player: Option<PlayerId>) {
        self.selected_player = player;
    }

    pub fn drag_end_position(&self) -> Option<FieldCoordinate> {
        self.drag_end_position
    }

    pub fn set_drag_end_position(&mut self, position: Option<FieldCoordinate>) {
        self.drag_end_position = position;
    }

    pub fn drag_start_position(&self) -> Option<FieldCoordinate> {
        self.drag_start_position
    }

    pub fn set_drag_start_position(&mut self, position: Option<FieldCoordinate>) {
        self.drag_start_position = position;
    }

    /// Java: `setBlockDiceResult` — clears then repopulates `blockRolls`.
    pub fn set_block_dice_result(&mut self, block_rolls: Vec<BlockRoll>) {
        self.clear_block_dice_result();
        self.block_rolls.extend(block_rolls);
    }

    pub fn clear_block_dice_result(&mut self) {
        self.block_rolls.clear();
    }

    pub fn block_rolls(&self) -> &[BlockRoll] {
        &self.block_rolls
    }

    pub fn set_status(&mut self, title: Option<String>, message: Option<String>, status_type: Option<StatusType>) {
        self.status_title = title;
        self.status_message = message;
        self.status_type = status_type;
    }

    pub fn clear_status(&mut self) {
        self.set_status(None, None, None);
    }

    pub fn status_title(&self) -> Option<&str> {
        self.status_title.as_deref()
    }

    pub fn status_message(&self) -> Option<&str> {
        self.status_message.as_deref()
    }

    pub fn status_type(&self) -> Option<StatusType> {
        self.status_type
    }

    pub fn set_acting_player_updated(&mut self, updated: bool) {
        self.acting_player_updated = updated;
    }

    pub fn is_acting_player_updated(&self) -> bool {
        self.acting_player_updated
    }

    pub fn set_turn_timer_stopped(&mut self, stopped: bool) {
        self.turn_timer_stopped = stopped;
    }

    pub fn is_turn_timer_stopped(&self) -> bool {
        self.turn_timer_stopped
    }

    pub fn spectator_count(&self) -> i32 {
        self.spectator_count
    }

    pub fn set_spectator_count(&mut self, count: i32) {
        self.spectator_count = count;
    }

    pub fn set_wizard_spell(&mut self, wizard_spell: Option<SpecialEffect>) {
        self.wizard_spell = wizard_spell;
    }

    pub fn wizard_spell(&self) -> Option<SpecialEffect> {
        self.wizard_spell
    }

    pub fn is_end_turn_button_hidden(&self) -> bool {
        self.end_turn_button_hidden
    }

    pub fn set_end_turn_button_hidden(&mut self, hidden: bool) {
        self.end_turn_button_hidden = hidden;
    }

    pub fn set_spectators(&mut self, spectators: Vec<String>) {
        self.spectators = spectators;
    }

    pub fn spectators(&self) -> &[String] {
        &self.spectators
    }

    pub fn coach_controlling_replay(&self) -> Option<&str> {
        self.coach_controlling_replay.as_deref()
    }

    pub fn set_coach_controlling_replay(&mut self, coach: Option<String>) {
        self.coach_controlling_replay = coach;
    }

    /// Java: `clear()`.
    pub fn clear(&mut self) {
        self.set_selected_player(None);
        self.set_drag_start_position(None);
        self.set_drag_end_position(None);
        self.clear_block_dice_result();
        self.clear_status();
        self.set_acting_player_updated(false);
        self.set_wizard_spell(None);
        self.set_end_turn_button_hidden(false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_has_no_selected_player() {
        assert!(ClientData::new().selected_player().is_none());
    }

    #[test]
    fn set_and_get_selected_player() {
        let mut data = ClientData::new();
        data.set_selected_player(Some("p1".to_string()));
        assert_eq!(data.selected_player(), Some(&"p1".to_string()));
    }

    #[test]
    fn set_block_dice_result_replaces_previous() {
        let mut data = ClientData::new();
        data.set_block_dice_result(vec![BlockRoll::default()]);
        assert_eq!(data.block_rolls().len(), 1);
        data.set_block_dice_result(vec![BlockRoll::default(), BlockRoll::default()]);
        assert_eq!(data.block_rolls().len(), 2);
    }

    #[test]
    fn clear_block_dice_result_empties_list() {
        let mut data = ClientData::new();
        data.set_block_dice_result(vec![BlockRoll::default()]);
        data.clear_block_dice_result();
        assert!(data.block_rolls().is_empty());
    }

    #[test]
    fn set_and_clear_status() {
        let mut data = ClientData::new();
        data.set_status(Some("t".into()), Some("m".into()), Some(StatusType::WAITING));
        assert_eq!(data.status_title(), Some("t"));
        data.clear_status();
        assert!(data.status_title().is_none());
        assert!(data.status_message().is_none());
        assert!(data.status_type().is_none());
    }

    #[test]
    fn acting_player_updated_flag_round_trips() {
        let mut data = ClientData::new();
        assert!(!data.is_acting_player_updated());
        data.set_acting_player_updated(true);
        assert!(data.is_acting_player_updated());
    }

    #[test]
    fn turn_timer_stopped_flag_round_trips() {
        let mut data = ClientData::new();
        data.set_turn_timer_stopped(true);
        assert!(data.is_turn_timer_stopped());
    }

    #[test]
    fn spectator_count_round_trips() {
        let mut data = ClientData::new();
        data.set_spectator_count(4);
        assert_eq!(data.spectator_count(), 4);
    }

    #[test]
    fn wizard_spell_round_trips() {
        let mut data = ClientData::new();
        data.set_wizard_spell(Some(SpecialEffect::FIREBALL));
        assert_eq!(data.wizard_spell(), Some(SpecialEffect::FIREBALL));
    }

    #[test]
    fn end_turn_button_hidden_round_trips() {
        let mut data = ClientData::new();
        data.set_end_turn_button_hidden(true);
        assert!(data.is_end_turn_button_hidden());
    }

    #[test]
    fn spectators_round_trip() {
        let mut data = ClientData::new();
        data.set_spectators(vec!["a".into(), "b".into()]);
        assert_eq!(data.spectators(), &["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn coach_controlling_replay_round_trips() {
        let mut data = ClientData::new();
        data.set_coach_controlling_replay(Some("coach".into()));
        assert_eq!(data.coach_controlling_replay(), Some("coach"));
    }

    #[test]
    fn clear_resets_all_transient_fields() {
        let mut data = ClientData::new();
        data.set_selected_player(Some("p1".into()));
        data.set_drag_start_position(Some(FieldCoordinate::new(1, 1)));
        data.set_drag_end_position(Some(FieldCoordinate::new(2, 2)));
        data.set_block_dice_result(vec![BlockRoll::default()]);
        data.set_status(Some("t".into()), Some("m".into()), Some(StatusType::WAITING));
        data.set_acting_player_updated(true);
        data.set_wizard_spell(Some(SpecialEffect::FIREBALL));
        data.set_end_turn_button_hidden(true);

        data.clear();

        assert!(data.selected_player().is_none());
        assert!(data.drag_start_position().is_none());
        assert!(data.drag_end_position().is_none());
        assert!(data.block_rolls().is_empty());
        assert!(data.status_title().is_none());
        assert!(!data.is_acting_player_updated());
        assert!(data.wizard_spell().is_none());
        assert!(!data.is_end_turn_button_hidden());
    }

    #[test]
    fn clear_does_not_reset_persistent_fields() {
        // Java's clear() intentionally leaves turnTimerStopped, spectatorCount, spectators,
        // and coachControllingReplay untouched — only re-verify per-selection state resets.
        let mut data = ClientData::new();
        data.set_turn_timer_stopped(true);
        data.set_spectator_count(3);
        data.clear();
        assert!(data.is_turn_timer_stopped());
        assert_eq!(data.spectator_count(), 3);
    }
}
