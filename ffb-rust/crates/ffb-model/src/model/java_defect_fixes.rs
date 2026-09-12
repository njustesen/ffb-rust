//! Opt-in corrections for defects in the **stock Java** engine.
//!
//! Java is the reference for this port: where Java disagrees with the printed rulebook, Java wins,
//! because matching Java is the whole point. A handful of cases are different in kind — Java does
//! something no reading of the rules supports and that we can demonstrate with a probe.
//!
//! Each one is ported 1:1 and is the DEFAULT here, so the two engines stay byte-comparable, with
//! the correction available behind the matching flag below. Every flag therefore defaults to
//! `false`, and turning one on is a deliberate divergence from stock Java: a parity gate run with
//! any flag enabled is not a parity measurement.
//!
//! Each flag is documented in `docs/JAVA_DEFECTS.md` under the JD number named in its doc comment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct JavaDefectFixes {
    /// **JD-001.** Clear `FieldModel::out_of_bounds` when a punt's distance roll lands ON the
    /// pitch.
    ///
    /// Java's `bb2025/punt/StepPuntDistance.executeStep` sets the flag when the punt leaves the
    /// pitch and clears it nowhere, so a distance RE-ROLL that lands in bounds leaves the flag set
    /// from the first roll. `leave()` then publishes a THROW_IN whose coordinate is an ordinary
    /// interior square, and `ThrowInMechanic.interpretThrowInDirectionRoll` — which handles only
    /// edge squares — throws `IllegalStateException: Unable to determine throwInDirection`. The
    /// exception is swallowed upstream, the step never sets a next action, and the game is stuck.
    ///
    /// With this `false` (the default) Rust reproduces all of that, ending the game through the
    /// driver's no-progress guard rather than a panic. With it `true` the flag is cleared and the
    /// punt resolves as the rules describe.
    pub punt_distance_clears_out_of_bounds: bool,
}

impl JavaDefectFixes {
    /// All defects ported, no corrections applied — byte-comparable with stock Java.
    pub fn faithful_to_java() -> Self { Self::default() }

    /// Every correction enabled. NOT parity-comparable against stock Java.
    pub fn all_fixed() -> Self {
        Self { punt_distance_clears_out_of_bounds: true }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The default MUST be the Java behaviour: a parity run that silently enabled a correction
    /// would be measuring Rust against a Java we are not running.
    #[test]
    fn default_is_faithful_to_java_with_every_fix_off() {
        let d = JavaDefectFixes::default();
        assert_eq!(d, JavaDefectFixes::faithful_to_java());
        assert!(!d.punt_distance_clears_out_of_bounds);
    }

    #[test]
    fn all_fixed_turns_every_flag_on() {
        let a = JavaDefectFixes::all_fixed();
        assert!(a.punt_distance_clears_out_of_bounds);
        assert_ne!(a, JavaDefectFixes::default());
    }
}
