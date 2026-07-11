use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use crate::enums::{CardEffect, Weather};
use crate::inducement::card::Card;
use crate::types::{FieldCoordinate, MoveSquare, PushbackSquare, RangeRuler};
use crate::enums::PlayerState;
use crate::model::player::PlayerId;
use crate::model::target_selection_state::TargetSelectionState;
use crate::model::dice_decoration::DiceDecoration;
use crate::marking::player_marker::PlayerMarker;
use crate::marking::field_marker::FieldMarker;

/// The game board: player positions, ball position, and transient UI state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldModel {
    pub weather: Weather,

    pub ball_coordinate: Option<FieldCoordinate>,
    pub ball_moving: bool,
    pub ball_in_play: bool,

    pub bomb_coordinate: Option<FieldCoordinate>,
    pub bomb_moving: bool,

    /// Java: FieldModel.fBallOutOfBounds — true when the ball lands out of bounds.
    pub out_of_bounds: bool,

    /// Maps player id → field coordinate. Absent = not on pitch.
    pub player_coordinates: HashMap<PlayerId, FieldCoordinate>,
    /// Maps player id → current PlayerState.
    pub player_states: HashMap<PlayerId, PlayerState>,

    pub range_ruler: Option<RangeRuler>,
    /// Java: FieldModel.fMoveSquares — keyed by coordinate for O(1) lookup via get_move_square.
    pub move_squares: HashMap<FieldCoordinate, MoveSquare>,
    pub pushback_squares: Vec<PushbackSquare>,

    /// Stadium trap door locations. Players landing here roll D6; on 1 they fall through.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub trap_doors: Vec<FieldCoordinate>,

    /// Java: FieldModel.fTargetSelectionState
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_selection_state: Option<TargetSelectionState>,

    /// Java: FieldModel.chomped — maps chomper player id → list of chompee player ids.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub chomped: HashMap<PlayerId, Vec<PlayerId>>,

    /// Java: FieldModel.fMultiBlockTargets — player ids selected as multi-block targets.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub multi_block_targets: Vec<PlayerId>,

    /// Java: FieldModel.fMultiBlockTargetCoordinates — coordinates of multi-block targets.
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub multi_block_target_coordinates: HashSet<FieldCoordinate>,

    /// Java: FieldModel.fBlockedForTricksterCoordinates — squares blocked for trickster.
    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub blocked_for_trickster_coordinates: HashSet<FieldCoordinate>,

    /// Java: FieldModel.fDiceDecorations — transient dice display decorations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dice_decorations: Vec<DiceDecoration>,

    /// Java: FieldModel.fCardEffectsByPlayerId — active card effects per player.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub card_effects: HashMap<PlayerId, HashSet<CardEffect>>,

    /// Java: addPrayerEnhancements / removePrayerEnhancements — prayer name → player IDs with that prayer active.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub prayer_enhancements: HashMap<String, HashSet<PlayerId>>,

    /// Java: FieldModel.fCardsByPlayerId — cards currently assigned to players.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub player_cards: HashMap<PlayerId, Vec<Card>>,

    /// Java: FieldModel.fPlayerMarkers — coach-authored text markers per player.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub player_markers: Vec<PlayerMarker>,

    /// Java: FieldModel.fFieldMarkers — coach-authored text markers per pitch square.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub field_markers: Vec<FieldMarker>,
}

impl FieldModel {
    pub fn new() -> Self {
        FieldModel {
            weather: Weather::Nice,
            ball_coordinate: None,
            ball_moving: false,
            ball_in_play: false,
            bomb_coordinate: None,
            bomb_moving: false,
            out_of_bounds: false,
            player_coordinates: HashMap::new(),
            player_states: HashMap::new(),
            range_ruler: None,
            move_squares: HashMap::new(),
            pushback_squares: Vec::new(),
            trap_doors: Vec::new(),
            target_selection_state: None,
            chomped: HashMap::new(),
            multi_block_targets: Vec::new(),
            multi_block_target_coordinates: HashSet::new(),
            blocked_for_trickster_coordinates: HashSet::new(),
            dice_decorations: Vec::new(),
            card_effects: HashMap::new(),
            prayer_enhancements: HashMap::new(),
            player_cards: HashMap::new(),
            player_markers: Vec::new(),
            field_markers: Vec::new(),
        }
    }

    /// True if there is a trap door at the given coordinate.
    pub fn has_trap_door(&self, coord: FieldCoordinate) -> bool {
        self.trap_doors.contains(&coord)
    }

    /// Java: FieldModel.add(TrapDoor) — places a trapdoor at the given coordinate.
    pub fn add_trap_door(&mut self, coord: FieldCoordinate) {
        if !self.trap_doors.contains(&coord) {
            self.trap_doors.push(coord);
        }
    }

    /// Java: FieldModel.clearTrapdoors() — removes all trapdoors from the field.
    pub fn clear_trap_doors(&mut self) {
        self.trap_doors.clear();
    }

    pub fn player_coordinate(&self, id: &str) -> Option<FieldCoordinate> {
        self.player_coordinates.get(id).copied()
    }

    pub fn player_state(&self, id: &str) -> Option<PlayerState> {
        self.player_states.get(id).copied()
    }

    pub fn set_player_state(&mut self, id: &str, state: PlayerState) {
        self.player_states.insert(id.to_owned(), state);
    }

    pub fn set_player_coordinate(&mut self, id: &str, coord: FieldCoordinate) {
        self.player_coordinates.insert(id.to_owned(), coord);
    }

    pub fn remove_player(&mut self, id: &str) {
        self.player_coordinates.remove(id);
        self.player_states.remove(id);
    }

    /// Java: FieldModel.addChomp — register chomper→chompee and set chomped bit on chompee's state.
    pub fn add_chomp(&mut self, chomper_id: &str, chompee_id: &str) {
        let chompees = self.chomped
            .entry(chomper_id.to_string())
            .or_default();
        if !chompees.contains(&chompee_id.to_string()) {
            chompees.push(chompee_id.to_string());
            if let Some(st) = self.player_states.get(chompee_id).copied() {
                self.player_states.insert(chompee_id.to_string(), st.change_chomped(true));
            }
        }
    }

    /// Java: FieldModel.removeChomps(Player chomper) — removes all chomps by chomper,
    /// clears the chomped state bit on each chompee that is no longer chomped by anyone.
    /// Returns map of chompeeId → true if chompee state was cleared, false if still chomped by another.
    pub fn remove_chomps(&mut self, chomper_id: &str) -> Vec<(String, bool)> {
        let chompees = match self.chomped.remove(chomper_id) {
            Some(v) => v,
            None => return vec![],
        };
        let mut result = Vec::new();
        for chompee_id in chompees {
            let still_chomped = self.chomped.values().any(|list| list.contains(&chompee_id));
            if !still_chomped {
                if let Some(st) = self.player_states.get(chompee_id.as_str()).copied() {
                    self.player_states.insert(chompee_id.clone(), st.change_chomped(false));
                }
                result.push((chompee_id, true));
            } else {
                result.push((chompee_id, false));
            }
        }
        result
    }

    /// Java: FieldModel.updateChomps(Player chomper) — removes any chomps where the chompee
    /// is no longer adjacent to the chomper.
    pub fn update_chomps(&mut self, chomper_id: &str) -> Vec<(String, bool)> {
        let chomper_coord = match self.player_coordinates.get(chomper_id).copied() {
            Some(c) => c,
            None => return self.remove_chomps(chomper_id),
        };
        let to_remove: Vec<String> = match self.chomped.get(chomper_id) {
            Some(chompees) => chompees.iter()
                .filter(|chompee_id| {
                    let adjacent = self.player_coordinates.get(chompee_id.as_str())
                        .map_or(false, |c| chomper_coord.is_adjacent(*c));
                    !adjacent
                })
                .cloned()
                .collect(),
            None => return vec![],
        };
        let mut result = Vec::new();
        for chompee_id in to_remove {
            let removed = self.remove_single_chomp(chomper_id, &chompee_id);
            result.push((chompee_id, removed));
        }
        result
    }

    /// Java: FieldModel.removeChomp(Player chomper, Player chompee) — remove a single chomp entry.
    /// Returns true if the chompee's state was cleared (no longer chomped by anyone).
    pub fn remove_single_chomp(&mut self, chomper_id: &str, chompee_id: &str) -> bool {
        if let Some(chompees) = self.chomped.get_mut(chomper_id) {
            if let Some(pos) = chompees.iter().position(|id| id == chompee_id) {
                chompees.remove(pos);
                let still_chomped = self.chomped.values().any(|list| list.contains(&chompee_id.to_string()));
                if !still_chomped {
                    if let Some(st) = self.player_states.get(chompee_id).copied() {
                        self.player_states.insert(chompee_id.to_string(), st.change_chomped(false));
                    }
                    return true;
                }
            }
        }
        false
    }

    pub fn player_at(&self, coord: FieldCoordinate) -> Option<&PlayerId> {
        self.player_coordinates
            .iter()
            .find(|(_, &c)| c == coord)
            .map(|(id, _)| id)
    }

    /// All player ids currently on the pitch (not in dugout).
    pub fn players_on_pitch(&self) -> impl Iterator<Item = &PlayerId> {
        self.player_coordinates
            .iter()
            .filter(|(_, coord)| coord.is_on_pitch())
            .map(|(id, _)| id)
    }

    /// Java: FieldModel.isMultiBlockTarget — returns true if the player is a multi-block target.
    pub fn is_multi_block_target(&self, player_id: &str) -> bool {
        self.multi_block_targets.iter().any(|id| id == player_id)
    }

    /// Java: FieldModel.wasMultiBlockTargetSquare — returns true if the coordinate was a multi-block target square.
    pub fn was_multi_block_target_square(&self, coord: FieldCoordinate) -> bool {
        self.multi_block_target_coordinates.contains(&coord)
    }

    /// Java: FieldModel.replaceMultiBlockTargetCoordinate — replaces old with new if present.
    pub fn replace_multi_block_target_coordinate(&mut self, old: FieldCoordinate, new: FieldCoordinate) {
        if self.multi_block_target_coordinates.remove(&old) {
            self.multi_block_target_coordinates.insert(new);
        }
    }

    /// Java: FieldModel.isBlockedForTrickster
    pub fn is_blocked_for_trickster(&self, coord: FieldCoordinate) -> bool {
        self.blocked_for_trickster_coordinates.contains(&coord)
    }

    /// Java: FieldModel.clearMultiBlockTargets — clears all multi-block state and resets player state bits.
    pub fn clear_multi_block_targets(&mut self) {
        let targets: Vec<PlayerId> = self.multi_block_targets.clone();
        for target in &targets {
            if let Some(state) = self.player_states.get(target).copied() {
                self.player_states.insert(
                    target.clone(),
                    state.change_selected_stab_target(false).change_selected_block_target(false),
                );
            }
        }
        self.multi_block_targets.clear();
        self.multi_block_target_coordinates.clear();
        self.blocked_for_trickster_coordinates.clear();
    }

    /// Java: FieldModel.clearDiceDecorations
    pub fn clear_dice_decorations(&mut self) {
        self.dice_decorations.clear();
    }

    /// Java: FieldModel.add(DiceDecoration)
    pub fn add_dice_decoration(&mut self, d: DiceDecoration) {
        self.dice_decorations.push(d);
    }

    /// Java: FieldModel.remove(DiceDecoration)
    pub fn remove_dice_decoration(&mut self, d: &DiceDecoration) -> bool {
        if let Some(pos) = self.dice_decorations.iter().position(|x| x == d) {
            self.dice_decorations.remove(pos);
            true
        } else {
            false
        }
    }

    /// Java: FieldModel.getDiceDecorations
    pub fn get_dice_decorations(&self) -> &[DiceDecoration] {
        &self.dice_decorations
    }

    /// Java: FieldModel.getDiceDecoration(FieldCoordinate) — find decoration at a coordinate.
    pub fn get_dice_decoration_at(&self, coord: FieldCoordinate) -> Option<&DiceDecoration> {
        self.dice_decorations.iter().find(|d| d.coordinate == Some(coord))
    }

    /// Java: FieldModel.clearMoveSquares
    pub fn clear_move_squares(&mut self) {
        self.move_squares.clear();
    }

    /// Java: FieldModel.getMoveSquare(FieldCoordinate)
    pub fn get_move_square(&self, coord: FieldCoordinate) -> Option<MoveSquare> {
        self.move_squares.get(&coord).copied()
    }

    /// Java: FieldModel.add(MoveSquare) — inserts with coordinate as key.
    pub fn add_move_square(&mut self, ms: MoveSquare) {
        self.move_squares.insert(ms.coordinate, ms);
    }

    /// Adjacent coordinates within the field bounds.
    pub fn adjacent_on_pitch(&self, coord: FieldCoordinate) -> Vec<FieldCoordinate> {
        coord
            .neighbours()
            .iter()
            .copied()
            .filter(|c| c.is_on_pitch())
            .collect()
    }

    /// Java: FieldModel.addCardEffect — adds a card effect to a player.
    pub fn add_card_effect(&mut self, player_id: &str, effect: CardEffect) {
        self.card_effects
            .entry(player_id.to_string())
            .or_default()
            .insert(effect);
    }

    /// Java: FieldModel.removeCardEffect — removes a card effect from a player. Returns true if removed.
    pub fn remove_card_effect(&mut self, player_id: &str, effect: CardEffect) -> bool {
        if let Some(effects) = self.card_effects.get_mut(player_id) {
            let removed = effects.remove(&effect);
            if effects.is_empty() {
                self.card_effects.remove(player_id);
            }
            removed
        } else {
            false
        }
    }

    /// Java: FieldModel.hasCardEffect — checks if a player has a specific card effect.
    pub fn has_card_effect(&self, player_id: &str, effect: CardEffect) -> bool {
        self.card_effects
            .get(player_id)
            .map_or(false, |set| set.contains(&effect))
    }

    /// Java: FieldModel.findPlayers(CardEffect) — returns all player ids with the given card effect.
    pub fn find_players_with_card_effect(&self, effect: CardEffect) -> Vec<&str> {
        self.card_effects
            .iter()
            .filter(|(_, effects)| effects.contains(&effect))
            .map(|(id, _)| id.as_str())
            .collect()
    }

    /// Java: FieldModel.addCard(Player, Card) — assigns a card to a player on the field.
    pub fn add_card(&mut self, player_id: &str, card: Card) {
        self.player_cards
            .entry(player_id.to_string())
            .or_default()
            .push(card);
    }

    /// Java: FieldModel.removeCard(Player, Card) — removes a card from a player.
    pub fn remove_card(&mut self, player_id: &str, card_name: &str) -> Option<Card> {
        if let Some(cards) = self.player_cards.get_mut(player_id) {
            if let Some(pos) = cards.iter().position(|c| c.name == card_name) {
                let removed = cards.remove(pos);
                if cards.is_empty() {
                    self.player_cards.remove(player_id);
                }
                return Some(removed);
            }
        }
        None
    }

    /// Java: FieldModel.getCards(Player) — returns all cards assigned to a player.
    pub fn get_cards(&self, player_id: &str) -> &[Card] {
        self.player_cards
            .get(player_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Java: FieldModel.findPlayer(Card) — finds which player holds a given card.
    pub fn find_player_with_card(&self, card_name: &str) -> Option<&str> {
        self.player_cards
            .iter()
            .find(|(_, cards)| cards.iter().any(|c| c.name == card_name))
            .map(|(id, _)| id.as_str())
    }

    /// Java: FieldModel.keepDeactivatedCard(Player, Card) — card stays on field after deactivation.
    pub fn keep_deactivated_card(&mut self, player_id: &str, card_name: &str) {
        // Card already in player_cards — mark it deactivated (it stays assigned).
        // In the headless engine the distinction doesn't affect game state.
        let _ = (player_id, card_name);
    }

    /// Java: FieldModel.addPrayerEnhancements(Player, Prayer) — marks the prayer as active on the player.
    pub fn add_prayer_enhancement(&mut self, player_id: &str, prayer_name: &str) {
        self.prayer_enhancements
            .entry(prayer_name.to_string())
            .or_default()
            .insert(player_id.to_string());
    }

    /// Java: FieldModel.removePrayerEnhancements(Player, Prayer) — removes the prayer from the player.
    pub fn remove_prayer_enhancement(&mut self, player_id: &str, prayer_name: &str) {
        if let Some(players) = self.prayer_enhancements.get_mut(prayer_name) {
            players.remove(player_id);
            if players.is_empty() {
                self.prayer_enhancements.remove(prayer_name);
            }
        }
    }

    /// Returns true if the given prayer is currently active on the given player.
    pub fn has_prayer_enhancement(&self, player_id: &str, prayer_name: &str) -> bool {
        self.prayer_enhancements
            .get(prayer_name)
            .map_or(false, |players| players.contains(player_id))
    }

    /// Returns all player IDs that currently have the given prayer active.
    pub fn find_players_with_prayer_enhancement(&self, prayer_name: &str) -> Vec<&str> {
        self.prayer_enhancements
            .get(prayer_name)
            .map(|ids| ids.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Removes all prayer enhancements for the given prayer from all players.
    pub fn clear_prayer_enhancement(&mut self, prayer_name: &str) {
        self.prayer_enhancements.remove(prayer_name);
    }

    /// Java: FieldModel.clearTrackNumbers() — clears the client-side movement-track visualisation.
    /// No game-state effect; track numbers are purely a UI concern.
    pub fn clear_track_numbers(&mut self) {
        // No-op: track numbers are client-side visualisation data not stored in the engine model.
    }

    /// Java: FieldModel.getPlayerMarkers()
    pub fn get_player_markers(&self) -> &[PlayerMarker] {
        &self.player_markers
    }

    /// Java: FieldModel.getPlayerMarker(String pPlayerId)
    pub fn get_player_marker(&self, player_id: &str) -> Option<&PlayerMarker> {
        self.player_markers
            .iter()
            .find(|m| m.get_player_id() == Some(player_id))
    }

    /// Java: FieldModel.add(PlayerMarker) — replaces any existing marker for the same player.
    pub fn add_player_marker(&mut self, player_marker: PlayerMarker) {
        let player_id = player_marker.get_player_id().map(|s| s.to_string());
        self.player_markers.retain(|m| m.get_player_id().map(|s| s.to_string()) != player_id);
        self.player_markers.push(player_marker);
    }

    /// Java: FieldModel.remove(PlayerMarker)
    pub fn remove_player_marker(&mut self, player_id: &str) -> bool {
        let before = self.player_markers.len();
        self.player_markers.retain(|m| m.get_player_id() != Some(player_id));
        self.player_markers.len() != before
    }

    /// Java: FieldModel.getFieldMarker(FieldCoordinate)
    pub fn get_field_marker(&self, coordinate: FieldCoordinate) -> Option<&FieldMarker> {
        self.field_markers
            .iter()
            .find(|m| m.get_coordinate() == Some(&coordinate))
    }

    /// Java: FieldModel.add(FieldMarker) — replaces any existing marker at the same coordinate.
    pub fn add_field_marker(&mut self, field_marker: FieldMarker) {
        let coordinate = field_marker.coordinate;
        self.field_markers.retain(|m| m.coordinate != coordinate);
        self.field_markers.push(field_marker);
    }

    /// Java: FieldModel.remove(FieldMarker)
    pub fn remove_field_marker(&mut self, coordinate: FieldCoordinate) -> bool {
        let before = self.field_markers.len();
        self.field_markers.retain(|m| m.get_coordinate() != Some(&coordinate));
        self.field_markers.len() != before
    }
}

impl Default for FieldModel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::{PlayerState, PS_STANDING};
    use crate::types::FieldCoordinate;
    use crate::model::block_kind::BlockKind;

    #[test]
    fn set_and_get_player_position() {
        let mut fm = FieldModel::new();
        let coord = FieldCoordinate::new(5, 7);
        fm.set_player_coordinate("p1", coord);
        fm.set_player_state("p1", PlayerState(0x00001));
        assert_eq!(fm.player_coordinate("p1"), Some(coord));
        assert_eq!(fm.player_state("p1").map(|s| s.0), Some(0x00001));
    }

    #[test]
    fn player_at_returns_id() {
        let mut fm = FieldModel::new();
        let coord = FieldCoordinate::new(10, 7);
        fm.set_player_coordinate("p1", coord);
        assert_eq!(fm.player_at(coord).map(|s| s.as_str()), Some("p1"));
        assert!(fm.player_at(FieldCoordinate::new(0, 0)).is_none());
    }

    #[test]
    fn serde_round_trip() {
        let fm = FieldModel::new();
        let json = serde_json::to_string(&fm).unwrap();
        let back: FieldModel = serde_json::from_str(&json).unwrap();
        assert_eq!(fm.weather, back.weather);
    }

    #[test]
    fn ball_coordinate_set_and_get() {
        let mut fm = FieldModel::new();
        assert!(fm.ball_coordinate.is_none());
        fm.ball_coordinate = Some(FieldCoordinate::new(8, 5));
        assert_eq!(fm.ball_coordinate, Some(FieldCoordinate::new(8, 5)));
    }

    #[test]
    fn ball_in_play_flag_toggled() {
        let mut fm = FieldModel::new();
        assert!(!fm.ball_in_play);
        fm.ball_in_play = true;
        assert!(fm.ball_in_play);
    }

    #[test]
    fn players_on_pitch_counts_correctly() {
        let mut fm = FieldModel::new();
        fm.set_player_coordinate("p1", FieldCoordinate::new(5, 7));
        fm.set_player_coordinate("p2", FieldCoordinate::new(10, 3));
        let count = fm.players_on_pitch().count();
        assert_eq!(count, 2);
    }

    #[test]
    fn player_state_missing_returns_none() {
        let fm = FieldModel::new();
        assert!(fm.player_state("nobody").is_none());
    }

    #[test]
    fn trap_doors_default_empty() {
        let fm = FieldModel::new();
        assert!(fm.trap_doors.is_empty());
        assert!(!fm.has_trap_door(FieldCoordinate::new(10, 7)));
    }

    #[test]
    fn has_trap_door_detects_coordinate() {
        let mut fm = FieldModel::new();
        let trap = FieldCoordinate::new(10, 7);
        fm.trap_doors.push(trap);
        assert!(fm.has_trap_door(trap));
        assert!(!fm.has_trap_door(FieldCoordinate::new(11, 7)));
    }

    #[test]
    fn add_trap_door_stores_coordinate() {
        let mut fm = FieldModel::new();
        let coord = FieldCoordinate::new(6, 1);
        fm.add_trap_door(coord);
        assert!(fm.has_trap_door(coord));
    }

    #[test]
    fn add_trap_door_no_duplicate() {
        let mut fm = FieldModel::new();
        let coord = FieldCoordinate::new(6, 1);
        fm.add_trap_door(coord);
        fm.add_trap_door(coord);
        assert_eq!(fm.trap_doors.len(), 1);
    }

    #[test]
    fn clear_trap_doors_empties_vec() {
        let mut fm = FieldModel::new();
        fm.add_trap_door(FieldCoordinate::new(6, 1));
        fm.add_trap_door(FieldCoordinate::new(19, 13));
        assert_eq!(fm.trap_doors.len(), 2);
        fm.clear_trap_doors();
        assert!(fm.trap_doors.is_empty());
    }

    #[test]
    fn remove_player_clears_position_and_state() {
        let mut fm = FieldModel::new();
        let coord = FieldCoordinate::new(5, 7);
        fm.set_player_coordinate("p1", coord);
        fm.set_player_state("p1", PlayerState(0x00001));
        assert!(fm.player_coordinate("p1").is_some());
        fm.remove_player("p1");
        assert!(fm.player_coordinate("p1").is_none());
        assert!(fm.player_state("p1").is_none());
    }

    #[test]
    fn clear_multi_block_targets_clears_all() {
        let mut fm = FieldModel::new();
        fm.multi_block_targets.push("p1".to_string());
        fm.multi_block_targets.push("p2".to_string());
        fm.multi_block_target_coordinates.insert(FieldCoordinate::new(5, 7));
        fm.blocked_for_trickster_coordinates.insert(FieldCoordinate::new(6, 7));

        fm.clear_multi_block_targets();

        assert!(fm.multi_block_targets.is_empty());
        assert!(fm.multi_block_target_coordinates.is_empty());
        assert!(fm.blocked_for_trickster_coordinates.is_empty());
    }

    #[test]
    fn clear_multi_block_targets_clears_player_state_bits() {
        let mut fm = FieldModel::new();
        let state = PlayerState(PS_STANDING)
            .change_selected_block_target(true)
            .change_selected_stab_target(true);
        fm.set_player_state("p1", state);
        fm.multi_block_targets.push("p1".to_string());

        fm.clear_multi_block_targets();

        let after = fm.player_state("p1").unwrap();
        assert!(!after.is_selected_block_target());
        assert!(!after.is_selected_stab_target());
    }

    #[test]
    fn clear_dice_decorations_removes_all() {
        let mut fm = FieldModel::new();
        fm.add_dice_decoration(DiceDecoration::new(FieldCoordinate::new(5, 7), 2, BlockKind::BLOCK));
        fm.add_dice_decoration(DiceDecoration::new(FieldCoordinate::new(6, 7), 1, BlockKind::STAB));
        assert_eq!(fm.get_dice_decorations().len(), 2);

        fm.clear_dice_decorations();
        assert!(fm.get_dice_decorations().is_empty());
    }

    #[test]
    fn add_and_get_dice_decoration() {
        let mut fm = FieldModel::new();
        let d = DiceDecoration::new(FieldCoordinate::new(3, 4), 3, BlockKind::BLOCK);
        fm.add_dice_decoration(d.clone());

        let decorations = fm.get_dice_decorations();
        assert_eq!(decorations.len(), 1);
        assert_eq!(decorations[0], d);
    }

    #[test]
    fn dice_decoration_at_finds_coord() {
        let mut fm = FieldModel::new();
        let coord = FieldCoordinate::new(7, 5);
        let d = DiceDecoration::new(coord, 2, BlockKind::CHAINSAW);
        fm.add_dice_decoration(d.clone());
        fm.add_dice_decoration(DiceDecoration::new(FieldCoordinate::new(1, 1), 1, BlockKind::BLOCK));

        let found = fm.get_dice_decoration_at(coord);
        assert!(found.is_some());
        assert_eq!(found.unwrap().nr_of_dice, 2);

        assert!(fm.get_dice_decoration_at(FieldCoordinate::new(0, 0)).is_none());
    }

    #[test]
    fn add_card_effect_stores_and_queries() {
        let mut fm = FieldModel::new();
        fm.add_card_effect("p1", CardEffect::Distracted);
        assert!(fm.has_card_effect("p1", CardEffect::Distracted));
        assert!(!fm.has_card_effect("p1", CardEffect::Sedative));
        assert!(!fm.has_card_effect("p2", CardEffect::Distracted));
    }

    #[test]
    fn remove_card_effect_clears_entry() {
        let mut fm = FieldModel::new();
        fm.add_card_effect("p1", CardEffect::Sedative);
        assert!(fm.remove_card_effect("p1", CardEffect::Sedative));
        assert!(!fm.has_card_effect("p1", CardEffect::Sedative));
        assert!(!fm.remove_card_effect("p1", CardEffect::Sedative));
    }

    #[test]
    fn find_players_with_card_effect_returns_affected() {
        let mut fm = FieldModel::new();
        fm.add_card_effect("p1", CardEffect::Distracted);
        fm.add_card_effect("p2", CardEffect::Distracted);
        fm.add_card_effect("p3", CardEffect::Sedative);
        let mut distracted = fm.find_players_with_card_effect(CardEffect::Distracted);
        distracted.sort();
        assert_eq!(distracted, vec!["p1", "p2"]);
        let sedated = fm.find_players_with_card_effect(CardEffect::Sedative);
        assert_eq!(sedated, vec!["p3"]);
    }

    #[test]
    fn add_card_assigns_to_player() {
        let mut fm = FieldModel::new();
        let card = Card::new("Chop Block", Some("CHOP_BLOCK"));
        fm.add_card("p1", card);
        let cards = fm.get_cards("p1");
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].get_name(), "Chop Block");
    }

    #[test]
    fn get_cards_empty_returns_empty_slice() {
        let fm = FieldModel::new();
        assert!(fm.get_cards("nobody").is_empty());
    }

    #[test]
    fn remove_card_removes_and_returns_it() {
        let mut fm = FieldModel::new();
        fm.add_card("p1", Card::new("Witch Brew", Some("WITCH_BREW")));
        fm.add_card("p1", Card::new("Distract", Some("DISTRACT")));
        let removed = fm.remove_card("p1", "Witch Brew");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().get_name(), "Witch Brew");
        assert_eq!(fm.get_cards("p1").len(), 1);
    }

    #[test]
    fn remove_card_unknown_returns_none() {
        let mut fm = FieldModel::new();
        assert!(fm.remove_card("p1", "No Such Card").is_none());
    }

    #[test]
    fn find_player_with_card_finds_correct_player() {
        let mut fm = FieldModel::new();
        fm.add_card("p1", Card::new("Chop Block", Some("CHOP_BLOCK")));
        fm.add_card("p2", Card::new("Distract", Some("DISTRACT")));
        assert_eq!(fm.find_player_with_card("Chop Block"), Some("p1"));
        assert_eq!(fm.find_player_with_card("Distract"), Some("p2"));
        assert!(fm.find_player_with_card("Unknown").is_none());
    }

    #[test]
    fn remove_card_removes_player_entry_when_empty() {
        let mut fm = FieldModel::new();
        fm.add_card("p1", Card::new("Chop Block", Some("CHOP_BLOCK")));
        fm.remove_card("p1", "Chop Block");
        assert!(fm.get_cards("p1").is_empty());
        assert!(fm.find_player_with_card("Chop Block").is_none());
    }

    #[test]
    fn player_markers_default_empty() {
        let fm = FieldModel::new();
        assert!(fm.get_player_markers().is_empty());
        assert!(fm.get_player_marker("p1").is_none());
    }

    #[test]
    fn add_player_marker_stores_and_replaces() {
        let mut fm = FieldModel::new();
        let mut m1 = crate::marking::player_marker::PlayerMarker::with_player_id("p1");
        m1.set_home_text("first");
        fm.add_player_marker(m1);
        assert_eq!(fm.get_player_marker("p1").and_then(|m| m.get_home_text()), Some("first"));

        let mut m2 = crate::marking::player_marker::PlayerMarker::with_player_id("p1");
        m2.set_home_text("second");
        fm.add_player_marker(m2);
        assert_eq!(fm.get_player_markers().len(), 1);
        assert_eq!(fm.get_player_marker("p1").and_then(|m| m.get_home_text()), Some("second"));
    }

    #[test]
    fn remove_player_marker_removes_entry() {
        let mut fm = FieldModel::new();
        fm.add_player_marker(crate::marking::player_marker::PlayerMarker::with_player_id("p1"));
        assert!(fm.remove_player_marker("p1"));
        assert!(fm.get_player_marker("p1").is_none());
        assert!(!fm.remove_player_marker("p1"));
    }

    #[test]
    fn field_markers_default_empty() {
        let fm = FieldModel::new();
        assert!(fm.get_field_marker(FieldCoordinate::new(1, 1)).is_none());
    }

    #[test]
    fn add_field_marker_stores_and_replaces() {
        let mut fm = FieldModel::new();
        let coord = FieldCoordinate::new(3, 3);
        let mut m1 = crate::marking::field_marker::FieldMarker::with_coordinate(coord);
        m1.set_home_text("first");
        fm.add_field_marker(m1);
        assert_eq!(fm.get_field_marker(coord).and_then(|m| m.get_home_text()), Some("first"));

        let mut m2 = crate::marking::field_marker::FieldMarker::with_coordinate(coord);
        m2.set_home_text("second");
        fm.add_field_marker(m2);
        assert_eq!(fm.field_markers.len(), 1);
        assert_eq!(fm.get_field_marker(coord).and_then(|m| m.get_home_text()), Some("second"));
    }

    #[test]
    fn remove_field_marker_removes_entry() {
        let mut fm = FieldModel::new();
        let coord = FieldCoordinate::new(4, 4);
        fm.add_field_marker(crate::marking::field_marker::FieldMarker::with_coordinate(coord));
        assert!(fm.remove_field_marker(coord));
        assert!(fm.get_field_marker(coord).is_none());
        assert!(!fm.remove_field_marker(coord));
    }
}
