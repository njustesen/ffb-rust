/// 1:1 translation of `com.fumbbl.ffb.injury.context.IInjuryContextModification`.
///
/// A marker trait for types that can modify an injury context.
pub trait IInjuryContextModification {
    /// Java: `IInjuryContextModification.requiresConditionalReRollSkill()`
    fn requires_conditional_re_roll_skill(&self) -> bool;
}
