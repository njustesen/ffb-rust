/// 1:1 translation of `com.fumbbl.ffb.report.logcontrol.SkipInjuryParts`.
///
/// Controls which parts of an injury report should be omitted (e.g. when showing
/// an apothecary re-roll that only reveals the modified injury roll, not the armour).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum SkipInjuryParts {
    /// Java: ARMOUR(true, false) — skip armour roll in report.
    Armour,
    /// Java: ARMOUR_AND_CAS(true, false, true) — skip armour + casualty rolls.
    ArmourAndCas,
    /// Java: ARMOUR_AND_INJURY(true, true) — two-arg ctor delegates to
    /// `this(armour, injury, injury)`, so cas defaults to true here too.
    ArmourAndInjury,
    /// Java: EVERYTHING_BUT_CAS(true, true, false) — skip armour + injury, keep casualty.
    EverythingButCas,
    /// Java: INJURY(false, true) — two-arg ctor delegates to `this(false, true, true)`,
    /// so this also skips the casualty roll (not injury only).
    Injury,
    /// Java: CAS(false, false, true) — skip casualty roll.
    Cas,
    /// Java: NONE(false, false) — show everything.
    None,
}

impl SkipInjuryParts {
    /// Java: isArmour() — should the armour roll be skipped?
    pub fn is_armour(self) -> bool {
        matches!(self,
            SkipInjuryParts::Armour |
            SkipInjuryParts::ArmourAndCas |
            SkipInjuryParts::ArmourAndInjury |
            SkipInjuryParts::EverythingButCas
        )
    }

    /// Java: isInjury() — should the injury roll be skipped?
    pub fn is_injury(self) -> bool {
        matches!(self,
            SkipInjuryParts::ArmourAndInjury |
            SkipInjuryParts::EverythingButCas |
            SkipInjuryParts::Injury
        )
    }

    /// Java: isCas() — should the casualty roll be skipped?
    ///
    /// Note: Java's two-arg enum constructor `SkipInjuryParts(armour, injury)` delegates
    /// to the three-arg one as `this(armour, injury, injury)`, so for variants declared
    /// with only two constructor args, `cas` defaults to the `injury` value (not `false`).
    /// This means `ARMOUR_AND_INJURY(true, true)` and `INJURY(false, true)` both skip
    /// casualty too, in addition to `ARMOUR_AND_CAS` and `CAS`.
    pub fn is_cas(self) -> bool {
        matches!(self,
            SkipInjuryParts::ArmourAndCas |
            SkipInjuryParts::ArmourAndInjury |
            SkipInjuryParts::Injury |
            SkipInjuryParts::Cas
        )
    }
}

impl std::fmt::Display for SkipInjuryParts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            SkipInjuryParts::Armour => "ARMOUR",
            SkipInjuryParts::ArmourAndCas => "ARMOUR_AND_CAS",
            SkipInjuryParts::ArmourAndInjury => "ARMOUR_AND_INJURY",
            SkipInjuryParts::EverythingButCas => "EVERYTHING_BUT_CAS",
            SkipInjuryParts::Injury => "INJURY",
            SkipInjuryParts::Cas => "CAS",
            SkipInjuryParts::None => "NONE",
        };
        write!(f, "{}", name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_skips_nothing() {
        assert!(!SkipInjuryParts::None.is_armour());
        assert!(!SkipInjuryParts::None.is_injury());
        assert!(!SkipInjuryParts::None.is_cas());
    }

    #[test]
    fn armour_and_cas_skips_armour_and_cas() {
        assert!(SkipInjuryParts::ArmourAndCas.is_armour());
        assert!(!SkipInjuryParts::ArmourAndCas.is_injury());
        assert!(SkipInjuryParts::ArmourAndCas.is_cas());
    }

    #[test]
    fn everything_but_cas_skips_armour_and_injury() {
        assert!(SkipInjuryParts::EverythingButCas.is_armour());
        assert!(SkipInjuryParts::EverythingButCas.is_injury());
        assert!(!SkipInjuryParts::EverythingButCas.is_cas());
    }

    #[test]
    fn injury_only_skips_injury() {
        assert!(!SkipInjuryParts::Injury.is_armour());
        assert!(SkipInjuryParts::Injury.is_injury());
        // Java's two-arg ctor `INJURY(false, true)` delegates to
        // `this(false, true, true)`, so cas defaults to the injury value (true).
        assert!(SkipInjuryParts::Injury.is_cas());
    }

    #[test]
    fn armour_and_injury_also_skips_cas() {
        // Java's two-arg ctor `ARMOUR_AND_INJURY(true, true)` delegates to
        // `this(true, true, true)`, so cas is also true, unlike EVERYTHING_BUT_CAS
        // which explicitly passes cas=false via the three-arg ctor.
        assert!(SkipInjuryParts::ArmourAndInjury.is_armour());
        assert!(SkipInjuryParts::ArmourAndInjury.is_injury());
        assert!(SkipInjuryParts::ArmourAndInjury.is_cas());
    }

    #[test]
    fn display_works() {
        assert_eq!(format!("{}", SkipInjuryParts::None), "NONE");
        assert_eq!(format!("{}", SkipInjuryParts::ArmourAndCas), "ARMOUR_AND_CAS");
    }
}
