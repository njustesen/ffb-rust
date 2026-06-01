use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use crate::enums::Weather;
use crate::types::{FieldCoordinate, PushbackSquare, RangeRuler};
use crate::enums::PlayerState;
use crate::model::player::PlayerId;

/// The game board: player positions, ball position, and transient UI state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldModel {
    pub weather: Weather,

    pub ball_coordinate: Option<FieldCoordinate>,
    pub ball_moving: bool,
    pub ball_in_play: bool,

    pub bomb_coordinate: Option<FieldCoordinate>,
    pub bomb_moving: bool,

    /// Maps player id → field coordinate. Absent = not on pitch.
    pub player_coordinates: HashMap<PlayerId, FieldCoordinate>,
    /// Maps player id → current PlayerState.
    pub player_states: HashMap<PlayerId, PlayerState>,

    pub range_ruler: Option<RangeRuler>,
    pub move_squares: HashSet<FieldCoordinate>,
    pub pushback_squares: Vec<PushbackSquare>,
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
            player_coordinates: HashMap::new(),
            player_states: HashMap::new(),
            range_ruler: None,
            move_squares: HashSet::new(),
            pushback_squares: Vec::new(),
        }
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
    use crate::enums::PlayerState;
    use crate::types::FieldCoordinate;

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
}
