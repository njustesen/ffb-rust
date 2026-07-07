use std::collections::HashMap;
use ffb_model::enums::TurnMode;
use ffb_model::types::FieldCoordinate;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandEndTurn`.
/// Sent when the active coach ends their team's turn.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandEndTurn {
    /// Java: `turnMode`
    pub turn_mode: Option<TurnMode>,
    /// Java: `playerCoordinates` — snapshot of player positions for client-side sync.
    pub player_coordinates: HashMap<String, FieldCoordinate>,
}

impl ClientCommandEndTurn {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_turn_mode(&self) -> Option<TurnMode> { self.turn_mode }
    pub fn get_player_coordinates(&self) -> &HashMap<String, FieldCoordinate> { &self.player_coordinates }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_empty() {
        let cmd = ClientCommandEndTurn::new();
        assert!(cmd.turn_mode.is_none());
        assert!(cmd.player_coordinates.is_empty());
    }

    #[test]
    fn turn_mode_stored() {
        let mut cmd = ClientCommandEndTurn::new();
        cmd.turn_mode = Some(TurnMode::Regular);
        assert_eq!(cmd.get_turn_mode(), Some(TurnMode::Regular));
    }

    #[test]
    fn player_coordinates_stored() {
        let mut cmd = ClientCommandEndTurn::new();
        cmd.player_coordinates.insert("p1".into(), FieldCoordinate::new(5, 5));
        assert_eq!(cmd.player_coordinates.len(), 1);
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandEndTurn::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandEndTurn::default().clone();
    }
}
