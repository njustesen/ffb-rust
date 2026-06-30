use serde::{Deserialize, Serialize};
use crate::enums::PlayerAction;
use crate::model::player::PlayerId;

/// Tracks the currently-acting player during a turn.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActingPlayer {
    pub player_id: Option<PlayerId>,
    pub player_action: Option<PlayerAction>,
    /// For Block/Blitz: the target defender chosen at activation time.
    pub defender_id: Option<PlayerId>,
    pub current_move: i32,
    pub goes_for_it: bool,
    pub standing_up: bool,
    pub jumping: bool,
    pub has_acted: bool,
    pub has_fouled: bool,
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
    /// Java: fHasMoved — true once the player has taken at least one move step.
    pub has_moved: bool,
    /// Java: fHasBlocked — true once the player has executed a block (StepBlockStatistics).
    pub has_blocked: bool,
    /// Java: fSufferingBloodLust — Vampire player must feed this turn or go frenzy.
    pub suffering_blood_lust: bool,
    /// Java: forgone — player chose to forgo their action (e.g. held by Take Root or flagged as stalling).
    pub forgone: bool,
    /// Java: fStrength — effective strength at block time (with modifiers; 0 = not set).
    pub strength: i32,
    /// Java: fellFromRush — true when a Ball & Chain player fell while going for it.
    pub fell_from_rush: bool,
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
        self.defender_id = None;
        self.current_move = 0;
        self.goes_for_it = false;
        self.standing_up = false;
        self.jumping = false;
        self.has_acted = false;
        self.held_in_place = false;
        self.suffering_blood_lust = false;
        self.forgone = false;
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

    #[test]
    fn set_player_stores_action_and_id() {
        let mut ap = ActingPlayer::new();
        ap.set_player("p42".into(), PlayerAction::Foul);
        assert_eq!(ap.player_id.as_deref(), Some("p42"));
        assert_eq!(ap.player_action, Some(PlayerAction::Foul));
    }

    #[test]
    fn set_player_resets_movement_counters() {
        let mut ap = ActingPlayer::new();
        ap.current_move = 5;
        ap.goes_for_it = true;
        ap.set_player("p1".into(), PlayerAction::Move);
        assert_eq!(ap.current_move, 0);
        assert!(!ap.goes_for_it);
    }

    #[test]
    fn special_flags_default_false() {
        let ap = ActingPlayer::new();
        assert!(!ap.jumping);
        assert!(!ap.standing_up);
        assert!(!ap.temporary_claws);
        assert!(!ap.temporary_frenzy);
        assert!(!ap.ignore_tackle_zones);
        assert_eq!(ap.fury_of_blood_god_blocks, 0);
    }
}
