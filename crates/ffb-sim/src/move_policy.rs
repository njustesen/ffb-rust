/// Tracks whether the active player may still move this activation.
///
/// Blood Bowl rule: a player may move once per "dice event" (any roll —
/// dodge, GFI, block, injury, pass, catch, pickup — or a new activation).
/// A second move without an intervening dice event forces the activation to end.
///
/// Ported from MovePolicyState.java (ffb-ai).
pub struct MovePolicyState {
    has_moved: bool,
    move_allowed: bool,
}

impl MovePolicyState {
    pub fn new() -> Self {
        Self {
            has_moved: false,
            move_allowed: true,
        }
    }

    /// Call when the player takes a move step.
    pub fn record_move(&mut self) {
        self.has_moved = true;
        self.move_allowed = false;
    }

    /// Call after any dice roll (dodge, block, injury, pass, catch, pickup, GFI, etc.).
    /// Re-opens the move window for one more step.
    pub fn reset(&mut self) {
        self.has_moved = false;
        self.move_allowed = true;
    }

    /// Call at the start of a new player activation. Equivalent to `reset`.
    pub fn new_activation(&mut self) {
        self.reset();
    }

    /// Returns `true` when the player has already moved and no dice event has
    /// reset the window — the activation must end.
    pub fn should_end_now(&self) -> bool {
        self.has_moved && !self.move_allowed
    }
}

impl Default for MovePolicyState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Basic state transitions ───────────────────────────────────────────────
    // Ported from MovePolicyStateTest.java

    #[test]
    fn initially_does_not_force_end() {
        let state = MovePolicyState::new();
        assert!(!state.should_end_now());
    }

    #[test]
    fn forces_end_after_move_without_dice() {
        let mut state = MovePolicyState::new();
        state.record_move();
        assert!(state.should_end_now());
    }

    #[test]
    fn dice_roll_resets_constraint() {
        let mut state = MovePolicyState::new();
        state.record_move();
        state.reset();
        assert!(!state.should_end_now());
    }

    #[test]
    fn multiple_dice_rolls_during_path_grants_only_one_move() {
        // Multiple dice dialogs (e.g. dodge + re-roll) during a single move path:
        // both resets don't pre-consume the next move window.
        let mut state = MovePolicyState::new();
        state.record_move();
        state.reset();
        state.reset(); // second dice event in the same move path
        assert!(!state.should_end_now()); // window still open
        state.record_move();
        assert!(state.should_end_now()); // second move locks it
    }

    #[test]
    fn new_activation_resets_constraint() {
        let mut state = MovePolicyState::new();
        state.record_move();
        state.new_activation();
        assert!(!state.should_end_now());
    }

    // ── Game sequence scenarios ───────────────────────────────────────────────

    #[test]
    fn blitz_move_block_dice_then_hit_and_run_move() {
        // Move to tackle zone → block dice (reset) → second move (HitAndRun).
        let mut state = MovePolicyState::new();
        state.record_move();
        assert!(state.should_end_now());
        state.reset(); // block dice rolled
        assert!(!state.should_end_now());
        state.record_move(); // HitAndRun second move
        assert!(state.should_end_now());
    }

    #[test]
    fn foul_move_then_injury_dice_then_reposition_move() {
        // Move to prone opponent → injury dice (reset) → reposition move.
        let mut state = MovePolicyState::new();
        state.record_move();
        assert!(state.should_end_now());
        state.reset(); // injury dice rolled
        assert!(!state.should_end_now());
        state.record_move();
        assert!(state.should_end_now());
    }

    #[test]
    fn pass_roll_then_give_and_go_move() {
        // Move → pass roll (reset) → catch roll (reset) → Give and Go move.
        let mut state = MovePolicyState::new();
        state.record_move();
        state.reset(); // pass roll
        state.reset(); // catch roll
        assert!(!state.should_end_now());
        state.record_move(); // Give and Go
        assert!(state.should_end_now());
    }

    #[test]
    fn hand_off_catch_roll_then_give_and_go_move() {
        // Move → catch roll for hand-off (reset) → Give and Go move.
        let mut state = MovePolicyState::new();
        state.record_move();
        state.reset(); // catch roll
        assert!(!state.should_end_now());
        state.record_move();
        assert!(state.should_end_now());
    }

    #[test]
    fn pick_up_roll_then_continue_running() {
        // Move to ball → pick-up roll (reset) → continue running.
        let mut state = MovePolicyState::new();
        state.record_move();
        state.reset(); // pick-up roll
        assert!(!state.should_end_now());
        state.record_move();
        assert!(state.should_end_now());
    }

    #[test]
    fn dodge_during_move_then_one_more_step() {
        // Enter tackle zone during move → dodge roll (reset) → one more step.
        let mut state = MovePolicyState::new();
        state.record_move();
        state.reset(); // dodge roll
        assert!(!state.should_end_now());
        state.record_move();
        assert!(state.should_end_now());
    }
}
