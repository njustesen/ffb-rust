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
    /// Java: hasTriggeredEffect — true once an out-of-activation effect (e.g. target-selection
    /// commitment) has been triggered for this player's turn.
    pub has_triggered_effect: bool,
    /// Java: fDodging — true while the player is in the middle of a dodge roll.
    pub dodging: bool,
    /// Java: fHasFed — Vampire has successfully fed this turn (BloodLust satisfied).
    pub has_fed: bool,
    /// Java: fHasPassed — true once the player has made a pass/hand-off this turn.
    pub has_passed: bool,
    /// Java: fSufferingAnimosity — true when the player failed an Animosity check
    /// and must re-select a passing target.
    pub suffering_animosity: bool,
    /// Java: mustCompleteAction — true once the player must complete their current
    /// action (e.g. `StepEndBomb` forces a second Ninja bomb throw) rather than being
    /// allowed to voluntarily end the turn.
    pub must_complete_action: bool,
    /// Java: fumblerooskiePending — true once CLIENT_USE_FUMBLEROOSKIE has been accepted
    /// (player action allows it and the player has the ball); cleared once the ball stops
    /// moving again. Consumed by `StepResetFumblerooskie`.
    pub fumblerooskie_pending: bool,
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
        self.must_complete_action = false;
    }

    pub fn is_active(&self) -> bool {
        self.player_id.is_some()
    }

    /// Java: `isMustCompleteAction()`.
    pub fn is_must_complete_action(&self) -> bool {
        self.must_complete_action
    }

    /// Java: `setMustCompleteAction(boolean)`.
    pub fn set_must_complete_action(&mut self, must_complete_action: bool) {
        self.must_complete_action = must_complete_action;
    }

    /// Java: `isFumblerooskiePending()`.
    pub fn is_fumblerooskie_pending(&self) -> bool {
        self.fumblerooskie_pending
    }

    /// Java: `setFumblerooskiePending(boolean)`.
    pub fn set_fumblerooskie_pending(&mut self, fumblerooskie_pending: bool) {
        self.fumblerooskie_pending = fumblerooskie_pending;
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
    fn must_complete_action_defaults_false_and_can_be_set() {
        let mut ap = ActingPlayer::new();
        assert!(!ap.is_must_complete_action());
        ap.set_must_complete_action(true);
        assert!(ap.is_must_complete_action());
    }

    #[test]
    fn set_player_resets_must_complete_action() {
        let mut ap = ActingPlayer::new();
        ap.set_must_complete_action(true);
        ap.set_player("p1".into(), PlayerAction::Move);
        assert!(!ap.is_must_complete_action());
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
