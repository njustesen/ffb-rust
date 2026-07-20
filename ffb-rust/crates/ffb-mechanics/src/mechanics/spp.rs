use ffb_model::enums::Rules;

/// SPP values awarded per action, by edition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SppTable {
    pub mvp: i32,
    pub touchdown: i32,
    pub casualty: i32,
    pub completion: i32,
    pub interception: i32,
    pub deflection: i32,
    pub catch: i32,
    pub landing: i32,
    /// Extra SPP for completion when team has the relevant special rule (BB2020/2025).
    pub additional_completion: i32,
    /// Extra SPP for casualty when team has the relevant special rule (BB2020/2025).
    pub additional_casualty: i32,
    /// Extra SPP for catch when team has the relevant special rule (BB2025).
    pub additional_catch: i32,
}

/// BB2016 SPP table.
pub const SPP_TABLE_BB2016: SppTable = SppTable {
    mvp: 5,
    touchdown: 3,
    casualty: 2,
    completion: 1,
    interception: 2,
    deflection: 1,
    catch: 1,
    landing: 0,
    additional_completion: 0,
    additional_casualty: 0,
    additional_catch: 0,
};

/// BB2020 SPP table.
pub const SPP_TABLE_BB2020: SppTable = SppTable {
    mvp: 4,
    touchdown: 3,
    casualty: 2,
    completion: 1,
    interception: 2,
    deflection: 1,
    catch: 1,
    landing: 0,
    additional_completion: 1,
    additional_casualty: 1,
    // Java: SppCalc.additionalSpp returns 1 for every non-BB2016 edition (catch included).
    additional_catch: 1,
};

/// BB2025 SPP table (base values; Brawlin' Brutes adjusts td/cas at call site).
pub const SPP_TABLE_BB2025: SppTable = SppTable {
    mvp: 4,
    touchdown: 3,
    casualty: 2,
    completion: 1,
    interception: 2,
    deflection: 1,
    catch: 1,
    landing: 1,
    additional_completion: 1,
    additional_casualty: 1,
    additional_catch: 1,
};

/// Return the SPP table for the given rules edition.
pub fn spp_table(rules: Rules) -> SppTable {
    match rules {
        Rules::Bb2016 => SPP_TABLE_BB2016,
        Rules::Bb2020 => SPP_TABLE_BB2020,
        Rules::Bb2025 | Rules::Common => SPP_TABLE_BB2025,
    }
}

/// Touchdown SPP, adjusted for BB2025 Brawlin' Brutes (2 instead of 3).
pub fn touchdown_spp(rules: Rules, has_brawlin_brutes: bool) -> i32 {
    if rules == Rules::Bb2025 && has_brawlin_brutes { 2 } else { spp_table(rules).touchdown }
}

/// Casualty SPP, adjusted for BB2025 Brawlin' Brutes (3 instead of 2).
pub fn casualty_spp(rules: Rules, has_brawlin_brutes: bool) -> i32 {
    if rules == Rules::Bb2025 && has_brawlin_brutes { 3 } else { spp_table(rules).casualty }
}

/// Level-up thresholds (same across all editions: 6/16/31/51/76/176).
pub const LEVEL_THRESHOLDS: [i32; 6] = [6, 16, 31, 51, 76, 176];

/// Current player level (0 = rookie, 1 = experienced, …) from cumulative SPP.
pub fn player_level(current_spp: i32) -> usize {
    LEVEL_THRESHOLDS.partition_point(|&t| current_spp >= t)
}

/// Whether the player just levelled up given old and new SPP totals.
pub fn just_levelled_up(old_spp: i32, new_spp: i32) -> bool {
    player_level(old_spp) < player_level(new_spp)
}

#[cfg(test)]
mod tests {
    // Test names mirror com.fumbbl.ffb.server.mechanic.SppCalcTest (Java camelCase →
    // snake_case), exercised against the ffb-mechanics SppTable API.
    use super::*;

    // ── Touchdown ─────────────────────────────────────────────────────────

    #[test]
    fn touchdown_bb2016_is3() {
        assert_eq!(touchdown_spp(Rules::Bb2016, false), 3);
    }

    #[test]
    fn touchdown_bb2020_is3() {
        assert_eq!(touchdown_spp(Rules::Bb2020, false), 3);
    }

    #[test]
    fn touchdown_bb2025_normal_team_is3() {
        assert_eq!(touchdown_spp(Rules::Bb2025, false), 3);
    }

    #[test]
    fn touchdown_bb2025_brawlin_brutes_is2() {
        assert_eq!(touchdown_spp(Rules::Bb2025, true), 2);
    }

    #[test]
    fn touchdown_bb2016_brawlin_brutes_has_no_effect() {
        // Brawlin' Brutes rule only exists in BB2025
        assert_eq!(touchdown_spp(Rules::Bb2016, true), 3);
    }

    #[test]
    fn touchdown_bb2020_brawlin_brutes_has_no_effect() {
        // Brawlin' Brutes rule only exists in BB2025
        assert_eq!(touchdown_spp(Rules::Bb2020, true), 3);
    }

    // ── Casualty ──────────────────────────────────────────────────────────

    #[test]
    fn casualty_bb2016_is2() {
        assert_eq!(casualty_spp(Rules::Bb2016, false), 2);
    }

    #[test]
    fn casualty_bb2020_is2() {
        assert_eq!(casualty_spp(Rules::Bb2020, false), 2);
    }

    #[test]
    fn casualty_bb2025_normal_team_is2() {
        assert_eq!(casualty_spp(Rules::Bb2025, false), 2);
    }

    #[test]
    fn casualty_bb2025_brawlin_brutes_is3() {
        assert_eq!(casualty_spp(Rules::Bb2025, true), 3);
    }

    // ── Constant events (same across all editions) ────────────────────────

    #[test]
    fn completion_is1_all_editions() {
        for rules in [Rules::Bb2016, Rules::Bb2020, Rules::Bb2025] {
            assert_eq!(spp_table(rules).completion, 1, "edition={rules:?}");
        }
    }

    #[test]
    fn interception_is2_all_editions() {
        for rules in [Rules::Bb2016, Rules::Bb2020, Rules::Bb2025] {
            assert_eq!(spp_table(rules).interception, 2, "edition={rules:?}");
        }
    }

    #[test]
    fn deflection_is1_all_editions() {
        for rules in [Rules::Bb2016, Rules::Bb2020, Rules::Bb2025] {
            assert_eq!(spp_table(rules).deflection, 1, "edition={rules:?}");
        }
    }

    #[test]
    fn catch_is1_all_editions() {
        for rules in [Rules::Bb2016, Rules::Bb2020, Rules::Bb2025] {
            assert_eq!(spp_table(rules).catch, 1, "edition={rules:?}");
        }
    }

    // ── Landing ───────────────────────────────────────────────────────────

    #[test]
    fn landing_bb2016_is0() {
        assert_eq!(spp_table(Rules::Bb2016).landing, 0);
    }

    #[test]
    fn landing_bb2020_is0() {
        assert_eq!(spp_table(Rules::Bb2020).landing, 0);
    }

    #[test]
    fn landing_bb2025_is1() {
        assert_eq!(spp_table(Rules::Bb2025).landing, 1);
    }

    // ── MVP ───────────────────────────────────────────────────────────────

    #[test]
    fn mvp_bb2016_is5() {
        assert_eq!(spp_table(Rules::Bb2016).mvp, 5);
    }

    #[test]
    fn mvp_bb2020_is4() {
        assert_eq!(spp_table(Rules::Bb2020).mvp, 4);
    }

    #[test]
    fn mvp_bb2025_is4() {
        assert_eq!(spp_table(Rules::Bb2025).mvp, 4);
    }

    // ── Additional SPP (league bonus; Java additionalSpp covers cas/comp) ──

    #[test]
    fn additional_spp_bb2016_is0() {
        assert_eq!(spp_table(Rules::Bb2016).additional_completion, 0);
        assert_eq!(spp_table(Rules::Bb2016).additional_casualty, 0);
    }

    #[test]
    fn additional_spp_bb2020_is1() {
        assert_eq!(spp_table(Rules::Bb2020).additional_completion, 1);
        assert_eq!(spp_table(Rules::Bb2020).additional_casualty, 1);
    }

    #[test]
    fn additional_spp_bb2025_is1() {
        assert_eq!(spp_table(Rules::Bb2025).additional_completion, 1);
        assert_eq!(spp_table(Rules::Bb2025).additional_casualty, 1);
    }

    // ── Level thresholds ──────────────────────────────────────────────────

    #[test]
    fn bb2016_level_thresholds_values() {
        assert_eq!(LEVEL_THRESHOLDS, [6, 16, 31, 51, 76, 176]);
    }

    #[test]
    fn bb2016_player_level_rookie() {
        assert_eq!(player_level(0), 0);
        assert_eq!(player_level(5), 0);
    }

    #[test]
    fn bb2016_player_level_experienced() {
        assert_eq!(player_level(6), 1);
        assert_eq!(player_level(15), 1);
    }

    #[test]
    fn bb2016_player_level_veteran() {
        assert_eq!(player_level(16), 2);
        assert_eq!(player_level(30), 2);
    }

    #[test]
    fn bb2016_player_level_emerging() {
        assert_eq!(player_level(31), 3);
        assert_eq!(player_level(50), 3);
    }

    #[test]
    fn bb2016_player_level_star() {
        assert_eq!(player_level(51), 4);
        assert_eq!(player_level(75), 4);
    }

    #[test]
    fn bb2016_player_level_super_star() {
        assert_eq!(player_level(76), 5);
        assert_eq!(player_level(175), 5);
    }

    #[test]
    fn bb2016_player_level_legend() {
        assert_eq!(player_level(176), 6);
        assert_eq!(player_level(999), 6);
    }

    #[test]
    fn bb2016_just_levelled_up_at_threshold() {
        assert!(just_levelled_up(5, 6)); // 0→1
        assert!(just_levelled_up(15, 16)); // 1→2
        assert!(just_levelled_up(30, 31)); // 2→3
        assert!(!just_levelled_up(6, 15)); // same level
        assert!(!just_levelled_up(16, 30)); // same level
    }

    // ── ffb-mechanics-specific extra (no Java SppCalc counterpart) ─────────
    // Java's additionalSpp() is a single value (1 for every non-BB2016
    // edition); the Rust table splits it per event, so all three additional_*
    // entries must match that value per edition.

    #[test]
    fn additional_catch_spp_by_edition() {
        assert_eq!(spp_table(Rules::Bb2016).additional_catch, 0);
        assert_eq!(spp_table(Rules::Bb2020).additional_catch, 1);
        assert_eq!(spp_table(Rules::Bb2025).additional_catch, 1);
    }
}
