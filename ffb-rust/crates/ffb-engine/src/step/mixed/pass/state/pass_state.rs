/// 1:1 translation of com.fumbbl.ffb.server.step.mixed.pass.state.PassState.
///
/// Holds transient pass-sequence state shared across multiple step files
/// (StepPass, StepIntercept, StepMissedPass, StepResolvePass, etc.).
use ffb_model::types::FieldCoordinate;
use ffb_model::enums::PassOutcome;
use ffb_model::enums::TurnMode;

pub struct PassState {
    /// Java: fCatcherId
    pub catcher_id: Option<String>,
    /// Java: fInterceptorId
    pub interceptor_id: Option<String>,
    /// Java: fOriginalBombardier
    pub original_bombardier: Option<String>,
    /// Java: fPassSkillUsed
    pub pass_skill_used: bool,
    /// Java: fInterceptorChosen
    pub interceptor_chosen: bool,
    /// Java: fDeflectionSuccessful
    pub deflection_successful: bool,
    /// Java: fInterceptionSuccessful
    pub interception_successful: bool,
    /// Java: fAllowMoveAfterBomb
    pub allow_move_after_bomb: bool,
    /// Java: fResult
    pub result: Option<PassOutcome>,
    /// Java: fThrowerCoordinate
    pub thrower_coordinate: Option<FieldCoordinate>,
    /// Java: fOldTurnMode
    pub old_turn_mode: Option<TurnMode>,
    /// Java: fUsingBlastIt (Java Boolean — nullable)
    pub using_blast_it: Option<bool>,
    /// Java: fThrowTwoBombs (Java Boolean — nullable)
    pub throw_two_bombs: Option<bool>,
}

impl PassState {
    pub fn new() -> Self {
        Self {
            catcher_id: None,
            interceptor_id: None,
            original_bombardier: None,
            pass_skill_used: false,
            interceptor_chosen: false,
            deflection_successful: false,
            interception_successful: false,
            allow_move_after_bomb: false,
            result: None,
            thrower_coordinate: None,
            old_turn_mode: None,
            using_blast_it: None,
            throw_two_bombs: None,
        }
    }

    /// Java: PassState.populate(PassState)
    ///
    /// Copies bombardier-related fields from another PassState (used when
    /// initialising for a new pass in a throw-two-bombs sequence).
    pub fn populate(&mut self, other: Option<&PassState>) {
        if let Some(o) = other {
            self.original_bombardier = o.original_bombardier.clone();
            self.throw_two_bombs = o.throw_two_bombs;
            self.allow_move_after_bomb = o.allow_move_after_bomb;
        }
    }

    /// Java: PassState.reset()
    ///
    /// Resets bombardier-related fields back to defaults between passes.
    pub fn reset(&mut self) {
        self.original_bombardier = None;
        self.throw_two_bombs = None;
        self.allow_move_after_bomb = false;
    }
}

impl Default for PassState {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_has_all_defaults() {
        let ps = PassState::new();
        assert!(ps.catcher_id.is_none());
        assert!(ps.interceptor_id.is_none());
        assert!(!ps.pass_skill_used);
        assert!(!ps.interceptor_chosen);
        assert!(!ps.deflection_successful);
        assert!(!ps.interception_successful);
        assert!(!ps.allow_move_after_bomb);
        assert!(ps.result.is_none());
        assert!(ps.thrower_coordinate.is_none());
        assert!(ps.old_turn_mode.is_none());
        assert!(ps.using_blast_it.is_none());
        assert!(ps.throw_two_bombs.is_none());
    }

    #[test]
    fn populate_copies_bombardier_fields() {
        let mut src = PassState::new();
        src.original_bombardier = Some("player1".to_string());
        src.throw_two_bombs = Some(true);
        src.allow_move_after_bomb = true;

        let mut dst = PassState::new();
        dst.populate(Some(&src));

        assert_eq!(dst.original_bombardier.as_deref(), Some("player1"));
        assert_eq!(dst.throw_two_bombs, Some(true));
        assert!(dst.allow_move_after_bomb);
        assert!(dst.catcher_id.is_none());
    }

    #[test]
    fn populate_with_none_is_noop() {
        let mut ps = PassState::new();
        ps.catcher_id = Some("x".to_string());
        ps.populate(None);
        assert_eq!(ps.catcher_id.as_deref(), Some("x"));
    }

    #[test]
    fn reset_clears_bombardier_fields() {
        let mut ps = PassState::new();
        ps.original_bombardier = Some("b".to_string());
        ps.throw_two_bombs = Some(false);
        ps.allow_move_after_bomb = true;
        ps.catcher_id = Some("c".to_string());

        ps.reset();

        assert!(ps.original_bombardier.is_none());
        assert!(ps.throw_two_bombs.is_none());
        assert!(!ps.allow_move_after_bomb);
        assert_eq!(ps.catcher_id.as_deref(), Some("c"));
    }
    #[test]
    fn populate_with_none_leaves_fields_unchanged() {
        let mut dst = PassState::new();
        dst.populate(None);
        assert!(dst.catcher_id.is_none());
        assert!(!dst.pass_skill_used);
    }
}
