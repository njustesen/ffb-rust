/// Re-export of the `SingleReRollUseState` type that lives in `abstract_step_multiple`.
///
/// Java: `com.fumbbl.ffb.server.step.mixed.SingleReRollUseState` — carries the player ID
/// chosen for a LORD_OF_CHAOS single-use team re-roll and the resolved re-roll source.
pub use crate::step::mixed::multiblock::abstract_step_multiple::SingleReRollUseState;

#[cfg(test)]
mod tests {
    use super::SingleReRollUseState;

    #[test]
    fn new_gives_all_none_fields() {
        let s = SingleReRollUseState::new();
        assert!(s.id.is_none());
        assert!(s.re_roll_source.is_none());
        assert!(s.re_roll_target.is_none());
    }

    #[test]
    fn default_gives_all_none_fields() {
        let s = SingleReRollUseState::default();
        assert!(s.id.is_none());
        assert!(s.re_roll_source.is_none());
        assert!(s.re_roll_target.is_none());
    }

    #[test]
    fn clone_works() {
        let mut s = SingleReRollUseState::new();
        s.id = Some("player-1".to_string());
        s.re_roll_source = Some("LORD_OF_CHAOS".to_string());
        let c = s.clone();
        assert_eq!(c.id, s.id);
        assert_eq!(c.re_roll_source, s.re_roll_source);
        assert!(c.re_roll_target.is_none());
    }
}
