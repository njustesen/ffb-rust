use crate::modifiers::gaze_modifier::GazeModifier;
use crate::modifiers::gaze_modifier_context::GazeModifierContext;
use crate::modifiers::modifier_type::ModifierType;

/// KNOWN GAP (out of scope for this file, documented here so it isn't mistaken for dead code
/// by mistake): this collection is never constructed by any caller in `ffb-engine`. The live
/// BB2020 Hypnotic Gaze step, `ffb-engine/src/step/bb2020/move_/step_hypnotic_gaze.rs`, hardcodes
/// `MINIMUM_ROLL_HYPNOTIC_GAZE = 3` and never builds a `GazeModifierContext`/queries this
/// collection, unlike Java's `StepHypnoticGaze.java` (bb2020) which calls
/// `GazeModifierFactory.findModifiers(...)` and feeds the tacklezone modifiers into
/// `AgilityMechanic.minimumRoll(player.getAgilityWithModifiers(), gazeModifiers)` — only BB2025's
/// `AgilityMechanic.minimumRollHypnoticGaze` truly hardcodes a flat 3. Fixing this requires
/// changing `step_hypnotic_gaze.rs` (outside the `modifiers/` audit scope of this pass), so it is
/// left as a documented, confirmed-but-unfixed gap rather than force-fitting a test against
/// currently-unreachable code.
pub struct GazeModifierCollection {
    modifiers: Vec<GazeModifier>,
}

impl GazeModifierCollection {
    pub fn new() -> Self {
        let mut col = Self { modifiers: Vec::new() };
        for i in 1i32..=8 {
            let name = if i == 1 { "1 Tacklezone".to_string() } else { format!("{} Tacklezones", i) };
            col.modifiers.push(GazeModifier::new_full(name, format!("{} for being marked", i), i - 1, i, ModifierType::TACKLEZONE));
        }
        col
    }

    pub fn get_modifiers(&self) -> &[GazeModifier] { &self.modifiers }
    pub fn find_applicable<'a>(&'a self, ctx: &GazeModifierContext<'_>) -> Vec<&'a GazeModifier> {
        self.modifiers.iter().filter(|m| m.applies_to_context(ctx)).collect()
    }
}

impl Default for GazeModifierCollection {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_eight_modifiers() {
        // 8 tacklezone modifiers (1..=8)
        assert_eq!(GazeModifierCollection::new().get_modifiers().len(), 8);
    }

    #[test]
    fn includes_single_tacklezone_modifier() {
        let col = GazeModifierCollection::new();
        assert!(col.get_modifiers().iter().any(|m| m.get_name() == "1 Tacklezone"));
    }

    #[test]
    fn all_modifiers_are_tacklezone_type() {
        let col = GazeModifierCollection::new();
        use crate::modifiers::modifier_type::ModifierType;
        assert!(col.get_modifiers().iter().all(|m| m.get_type() == ModifierType::TACKLEZONE));
    }

    #[test]
    fn includes_eight_tacklezones_modifier() {
        let col = GazeModifierCollection::new();
        assert!(col.get_modifiers().iter().any(|m| m.get_name() == "8 Tacklezones"));
    }

    #[test]
    fn tacklezone_count_is_eight() {
        let col = GazeModifierCollection::new();
        use crate::modifiers::modifier_type::ModifierType;
        let tz_count = col.get_modifiers().iter().filter(|m| m.get_type() == ModifierType::TACKLEZONE).count();
        assert_eq!(tz_count, 8);
    }
}
