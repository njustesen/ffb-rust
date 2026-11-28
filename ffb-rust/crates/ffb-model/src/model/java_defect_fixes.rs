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

    /// **JD-002.** Let the referee step survive a foul whose injury context has NO armour dice.
    ///
    /// Java's `SneakyGitBehaviour` StepReferee hook (all three editions) reads
    /// `injuryContext().getArmorRoll()` whenever the fouler lacks Sneaky Git OR the armour is
    /// broken. A Ball & Chain victim's armour is marked broken WITHOUT a roll
    /// (`UtilServerInjury.handleInjury`, `placedProneCausesInjuryRoll`), so the array is null and
    /// the hook throws `NullPointerException` out of `StepReferee.executeStep`. Nothing catches it
    /// in the stock engine: the game ends right there, in the state it had before the step ran.
    ///
    /// With this `false` (the default) Rust reproduces that outcome — the step returns without
    /// touching anything and the driver's no-progress guard ends the game where Java's crash does.
    /// With it `true` "no armour dice" reads as "no armour doubles" and the referee goes on to the
    /// injury dice, the only reading the rules support.
    pub referee_survives_missing_armour_roll: bool,

    /// **JD-004.** bb2020: spend a leap's team re-roll ONCE.
    ///
    /// Java's bb2020 `StepJump.executeStep` has no `usingDivingTackle == null` guard (bb2025's
    /// has one): after a re-rolled leap succeeds and the opposing coach answers the Diving Tackle
    /// question, the step runs again with JUMP still the re-rolled action and spends the re-roll a
    /// second time (a team with no re-rolls left goes to -1). `true`: the answer spends nothing.
    pub leap_reroll_spent_once: bool,

    /// **JD-005.** bb2025: the Arm Bar choice does not re-run the dodge re-roll.
    ///
    /// Java's bb2025 `StepMoveDodge`: a dodge fails, is re-rolled and fails again next to two or
    /// more Arm Bar players; the opposing coach's ARM_BAR choice re-runs `executeStep` with DODGE
    /// still the re-rolled action, so the re-roll is spent again and a third dodge die is rolled
    /// (which can turn the failed dodge into a success). `true`: the choice only names the Arm Bar
    /// player and the dodge stays failed.
    pub arm_bar_choice_keeps_failed_dodge: bool,

    /// **JD-006.** Survive a Furious Outburst that ends without a target selection.
    ///
    /// Java's `StepInitFuriousOutburst` reaches END without a `TargetSelectionState` when the coach
    /// ends the turn at the target prompt (also: prone/stunned player, no eligible target), and
    /// `StepEndFuriousOutburst` dereferences it: `NullPointerException`, the game stops there.
    /// `false` (default): Rust stops the same way (the step returns having touched nothing, as
    /// JD-002). `true`: "no selection" reads as "no target" and the activation ends normally.
    pub furious_outburst_survives_missing_target: bool,

    /// **JD-007.** Interpret the re-rolled (alternate) injury context, not the original.
    ///
    /// Java's `ModificationAwareInjuryTypeServer`: when a skill re-rolls a failed armour roll
    /// (bb2025 Master Assassin on a Stab, Old Pro), eleven injury types finish `injuryRoll` with
    /// the context-less `setInjury`, which interprets the ORIGINAL context: a needless casualty roll
    /// is made for it (its armour held), and the re-rolled context's injury is never interpreted,
    /// so the victim is not hurt. `true`: the injury is interpreted on the context it was rolled on.
    pub modified_injury_interprets_alternate_context: bool,

    /// **JD-008.** bb2025: a Pitch Invasion applies the Ball & Chain player's injury.
    ///
    /// Java's `UtilServerInjury.dropPlayer` rolls the chain injury and only PUBLISHES it as
    /// INJURY_RESULT, and the bb2025 kick-off sequence has no step that applies it: the Fanatic is
    /// reported stunned and stays standing. `true`: the result is applied as the bb2020 kick-off
    /// sequence's apothecary step would.
    pub pitch_invasion_applies_chain_injury: bool,
}

impl JavaDefectFixes {
    /// All defects ported, no corrections applied — byte-comparable with stock Java.
    pub fn faithful_to_java() -> Self { Self::default() }

    /// Every correction enabled. NOT parity-comparable against stock Java.
    pub fn all_fixed() -> Self {
        Self {
            punt_distance_clears_out_of_bounds: true,
            referee_survives_missing_armour_roll: true,
            leap_reroll_spent_once: true,
            arm_bar_choice_keeps_failed_dodge: true,
            furious_outburst_survives_missing_target: true,
            modified_injury_interprets_alternate_context: true,
            pitch_invasion_applies_chain_injury: true,
        }
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
        assert!(!d.referee_survives_missing_armour_roll);
        assert!(!d.leap_reroll_spent_once);
        assert!(!d.arm_bar_choice_keeps_failed_dodge);
        assert!(!d.furious_outburst_survives_missing_target);
        assert!(!d.modified_injury_interprets_alternate_context);
        assert!(!d.pitch_invasion_applies_chain_injury);
    }

    #[test]
    fn all_fixed_turns_every_flag_on() {
        let a = JavaDefectFixes::all_fixed();
        assert!(a.punt_distance_clears_out_of_bounds);
        assert!(a.referee_survives_missing_armour_roll);
        assert!(a.leap_reroll_spent_once);
        assert!(a.arm_bar_choice_keeps_failed_dodge);
        assert!(a.furious_outburst_survives_missing_target);
        assert!(a.modified_injury_interprets_alternate_context);
        assert!(a.pitch_invasion_applies_chain_injury);
        assert_ne!(a, JavaDefectFixes::default());
    }
}
