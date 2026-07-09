/// 1:1 translation of com.fumbbl.ffb.model.skill.DeclareCondition.
///
/// NOTE: The canonical enum lives in `crate::enums::DeclareCondition` (enums/skill.rs).
/// This file re-exports it for code that imports from `model::skill::declare_condition`.
pub use crate::enums::DeclareCondition;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_is_always_fulfilled() {
        assert!(DeclareCondition::None.fulfilled(true));
        assert!(DeclareCondition::None.fulfilled(false));
    }

    #[test]
    fn standing_requires_standing() {
        assert!(DeclareCondition::Standing.fulfilled(true));
        assert!(!DeclareCondition::Standing.fulfilled(false));
    }
}
