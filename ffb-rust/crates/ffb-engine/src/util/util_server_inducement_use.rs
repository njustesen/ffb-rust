// 1:1 translation of com.fumbbl.ffb.server.util.UtilServerInducementUse.
//
// Java source has two overloads of `useInducement`:
//   1. (GameState, Team, InducementType, int) — resolves which InducementSet to use
//      then delegates to overload 2.
//   2. (InducementType, int, InducementSet)   — core accounting logic.
//
// The Rust `InducementSet` model is still a stub (no fields).  We translate the
// core record-keeping logic using explicit `value` and `already_used` arguments
// that callers supply until the full InducementSet is implemented.
//
// Translated:
//   use_inducement(value, already_used, nr_of_uses) -> bool  — core charge accounting
//   use_one(value, already_used) -> bool                     — convenience wrapper
//
// The two-arg overload that resolves the correct InducementSet from a Game + Team
// is left as a stub because InducementSet and TurnData.inducement_set are not yet
// modelled in this version.

pub struct UtilServerInducementUse;

impl UtilServerInducementUse {
    /// Java: UtilServerInducementUse.useInducement(InducementType, int, InducementSet)
    ///
    /// Attempts to consume `nr_of_uses` charges of an inducement that currently
    /// has `value` total charges and `already_used` charges already spent.
    ///
    /// Returns `true` and increments `*already_used` if sufficient charges remain;
    /// returns `false` and leaves `*already_used` unchanged otherwise.
    ///
    /// Direct translation of Java:
    ///   if ((inducement.getValue() - inducement.getUses()) >= pNrOfUses) {
    ///       inducement.setUses(inducement.getUses() + pNrOfUses);
    ///   }
    pub fn use_inducement(value: i32, already_used: &mut i32, nr_of_uses: i32) -> bool {
        if (value - *already_used) >= nr_of_uses {
            *already_used += nr_of_uses;
            true
        } else {
            false
        }
    }

    /// Convenience: use exactly 1 charge.
    pub fn use_one(value: i32, already_used: &mut i32) -> bool {
        Self::use_inducement(value, already_used, 1)
    }
}

impl Default for UtilServerInducementUse {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn use_inducement_succeeds_when_charges_remain() {
        let mut used = 0;
        let result = UtilServerInducementUse::use_inducement(3, &mut used, 1);
        assert!(result);
        assert_eq!(used, 1);
    }

    #[test]
    fn use_inducement_fails_when_no_charges_remain() {
        let mut used = 3;
        let result = UtilServerInducementUse::use_inducement(3, &mut used, 1);
        assert!(!result);
        assert_eq!(used, 3); // unchanged
    }

    #[test]
    fn use_inducement_succeeds_for_multiple_charges() {
        let mut used = 1;
        let result = UtilServerInducementUse::use_inducement(3, &mut used, 2);
        assert!(result);
        assert_eq!(used, 3);
    }

    #[test]
    fn use_inducement_fails_when_not_enough_for_multiple_charges() {
        let mut used = 2;
        let result = UtilServerInducementUse::use_inducement(3, &mut used, 2);
        assert!(!result);
        assert_eq!(used, 2);
    }

    #[test]
    fn use_inducement_exactly_one_charge_left_succeeds() {
        let mut used = 2;
        let result = UtilServerInducementUse::use_inducement(3, &mut used, 1);
        assert!(result);
        assert_eq!(used, 3);
    }

    #[test]
    fn use_one_is_equivalent_to_use_inducement_with_1() {
        let mut used_a = 0;
        let mut used_b = 0;
        let r_a = UtilServerInducementUse::use_one(2, &mut used_a);
        let r_b = UtilServerInducementUse::use_inducement(2, &mut used_b, 1);
        assert_eq!(r_a, r_b);
        assert_eq!(used_a, used_b);
    }

    #[test]
    fn use_inducement_zero_value_always_fails() {
        let mut used = 0;
        let result = UtilServerInducementUse::use_inducement(0, &mut used, 1);
        assert!(!result);
        assert_eq!(used, 0);
    }

    #[test]
    fn use_inducement_nr_of_uses_zero_always_succeeds() {
        // Consuming 0 uses always succeeds (vacuously), similar to Java.
        let mut used = 5;
        let result = UtilServerInducementUse::use_inducement(5, &mut used, 0);
        assert!(result);
        assert_eq!(used, 5); // unchanged since we added 0
    }
}
