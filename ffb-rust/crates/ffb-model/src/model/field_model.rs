use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use crate::enums::Weather;
use crate::types::{FieldCoordinate, PushbackSquare, RangeRuler};
use crate::enums::PlayerState;
use crate::model::player::PlayerId;
use crate::model::target_selection_state::TargetSelectionState;
use crate::model::dice_decoration::DiceDecoration;

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
    pub move_squares: HashSet<FieldCoordinate>,
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
            move_squares: HashSet::new(),
            pushback_squares: Vec::new(),
            trap_doors: Vec::new(),
            target_selection_state: None,
            chomped: HashMap::new(),
            multi_block_targets: Vec::new(),
            multi_block_target_coordinates: HashSet::new(),
            blocked_for_trickster_coordinates: HashSet::new(),
            dice_decorations: Vec::new(),
        }
    }

    /// True if there is a trap door at the given coordinate.
    pub fn has_trap_door(&self, coord: FieldCoordinate) -> bool {
        self.trap_doors.contains(&coord)
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

    /// Adjacent coordinates within the field bounds.
    pub fn adjacent_on_pitch(&self, coord: FieldCoordinate) -> Vec<FieldCoordinate> {
        coord
            .neighbours()
            .iter()
            .copied()
            .filter(|c| c.is_on_pitch())
            .collect()
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
}
