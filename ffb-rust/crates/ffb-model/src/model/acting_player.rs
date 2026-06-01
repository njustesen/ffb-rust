use serde::{Deserialize, Serialize};
use crate::enums::PlayerAction;
use crate::model::player::PlayerId;

/// Tracks the currently-acting player during a turn.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActingPlayer {
    pub player_id: Option<PlayerId>,
    pub player_action: Option<PlayerAction>,
    pub current_move: i32,
    pub goes_for_it: bool,
    pub standing_up: bool,
    pub jumping: bool,
    pub has_acted: bool,
    pub held_in_place: bool,
    /// For Blitz/StandUpBlitz activations: true once the block step has been executed.
    pub blitz_blocked: bool,
    /// SlashingNails: player temporarily gains Claws for this blitz activation.
    pub temporary_claws: bool,
    /// FrenziedRush: player temporarily gains Frenzy for this blitz activation.
    pub temporary_frenzy: bool,
    /// Incorporeal: player ignores tackle zones when moving this activation (no dodge rolls).
    pub ignore_tackle_zones: bool,
    /// FuryOfTheBloodGod: extra block actions remaining (0 = not active).
    pub fury_of_blood_god_blocks: u8,
}

impl ActingPlayer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn set_player(&mut self, id: PlayerId, action: PlayerAction) {
        self.player_id = Some(id);
        self.player_action = Some(action);
        self.current_move = 0;
        self.goes_for_it = false;
        self.standing_up = false;
        self.jumping = false;
        self.has_acted = false;
        self.held_in_place = false;
    }

    pub fn is_active(&self) -> bool {
        self.player_id.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::PlayerAction;

    #[test]
    fn default_is_not_active() {
        let ap = ActingPlayer::new();
        assert!(!ap.is_active());
    }

    #[test]
    fn set_and_clear() {
        let mut ap = ActingPlayer::new();
        ap.set_player("p1".into(), PlayerAction::Move);
        assert!(ap.is_active());
        ap.clear();
        assert!(!ap.is_active());
    }

    #[test]
    fn serde_round_trip() {
        let mut ap = ActingPlayer::new();
        ap.set_player("p1".into(), PlayerAction::Blitz);
        let json = serde_json::to_string(&ap).unwrap();
        let back: ActingPlayer = serde_json::from_str(&json).unwrap();
        assert_eq!(ap.player_id, back.player_id);
    }
}
