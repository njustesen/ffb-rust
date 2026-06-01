package com.fumbbl.ffb.server.util;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class SpecialRollCalcTest {

    // ── minimumRollDauntless ──────────────────────────────────────────────────

    @Test
    void dauntless_str3vs4_minimum2() {
        assertEquals(2, SpecialRollCalc.minimumRollDauntless(3, 4));
    }

    @Test
    void dauntless_str2vs6_cappedAt6() {
        // 6 - 2 + 1 = 5, not capped
        assertEquals(5, SpecialRollCalc.minimumRollDauntless(2, 6));
    }

    @Test
    void dauntless_str1vs6_is6() {
        // 6 - 1 + 1 = 6
        assertEquals(6, SpecialRollCalc.minimumRollDauntless(1, 6));
    }

    @Test
    void dauntless_str3vs8_cappedAt6() {
        // 8 - 3 + 1 = 6, capped at 6
        assertEquals(6, SpecialRollCalc.minimumRollDauntless(1, 100));
    }

    // ── minimumRollTentaclesEscape ────────────────────────────────────────────

    @Test
    void tentacles_equal_strength_minimum6() {
        assertEquals(6, SpecialRollCalc.minimumRollTentaclesEscape(3, 3));
    }

    @Test
    void tentacles_stronger_opponent_higherMinimum() {
        assertEquals(8, SpecialRollCalc.minimumRollTentaclesEscape(5, 3));
    }

    @Test
    void tentacles_escape_success() {
        assertTrue(SpecialRollCalc.isTentaclesEscapeSuccessful(4, 4, 3, 3));   // 8 >= 6
    }

    @Test
    void tentacles_escape_failure() {
        assertFalse(SpecialRollCalc.isTentaclesEscapeSuccessful(2, 3, 5, 3));  // 5 < 8
    }

    // ── minimumRollShadowingEscape ────────────────────────────────────────────

    @Test
    void shadowing_equal_movement_minimum8() {
        assertEquals(8, SpecialRollCalc.minimumRollShadowingEscape(4, 4));
    }

    @Test
    void shadowing_faster_shadow_higherMinimum() {
        assertEquals(10, SpecialRollCalc.minimumRollShadowingEscape(6, 4));
    }

    @Test
    void shadowing_escape_success() {
        assertTrue(SpecialRollCalc.isShadowingEscapeSuccessful(5, 5, 4, 4));   // 10 >= 8
    }

    @Test
    void shadowing_escape_failure() {
        assertFalse(SpecialRollCalc.isShadowingEscapeSuccessful(2, 3, 6, 4));  // 5 < 10
    }

    // ── constant minimum rolls ────────────────────────────────────────────────

    @Test
    void chainsaw_minimum_is_2() {
        assertEquals(2, SpecialRollCalc.minimumRollChainsaw());
    }

    @Test
    void foulAppearance_minimum_is_2() {
        assertEquals(2, SpecialRollCalc.minimumRollResistingFoulAppearance());
    }

    @Test
    void confusion_good_conditions_is_2() {
        assertEquals(2, SpecialRollCalc.minimumRollConfusion(true));
    }

    @Test
    void confusion_bad_conditions_is_4() {
        assertEquals(4, SpecialRollCalc.minimumRollConfusion(false));
    }

    @Test
    void bloodLust_minimum_is_2() {
        assertEquals(2, SpecialRollCalc.minimumRollBloodLust());
    }

    @Test
    void animosity_minimum_is_2() {
        assertEquals(2, SpecialRollCalc.minimumRollAnimosity());
    }

    // ── success checks ────────────────────────────────────────────────────────

    @Test
    void regeneration_4plus_succeeds() {
        assertFalse(SpecialRollCalc.isRegenerationSuccessful(3));
        assertTrue(SpecialRollCalc.isRegenerationSuccessful(4));
        assertTrue(SpecialRollCalc.isRegenerationSuccessful(6));
    }

    @Test
    void pitchInvasion_1_always_safe() {
        assertFalse(SpecialRollCalc.isAffectedByPitchInvasion(1, 3));
    }

    @Test
    void pitchInvasion_roll2_fame3_is5_notAffected() {
        assertFalse(SpecialRollCalc.isAffectedByPitchInvasion(2, 3));  // 2+3=5 < 6
    }

    @Test
    void pitchInvasion_roll3_fame3_is6_affected() {
        assertTrue(SpecialRollCalc.isAffectedByPitchInvasion(3, 3));   // 3+3=6
    }

    @Test
    void knockout_recovery_1_never_recovers() {
        assertFalse(SpecialRollCalc.isRecoveringFromKnockout(1, 3));
    }

    @Test
    void knockout_recovery_2_with_2babes_succeeds() {
        assertTrue(SpecialRollCalc.isRecoveringFromKnockout(2, 2));    // 2+2=4 > 3
    }

    @Test
    void knockout_recovery_2_with_0babes_fails() {
        assertFalse(SpecialRollCalc.isRecoveringFromKnockout(2, 0));   // 2 not > 3
    }

    @Test
    void alwaysHungry_1_fails() {
        assertFalse(SpecialRollCalc.isAlwaysHungrySuccessful(1));
    }

    @Test
    void alwaysHungry_2plus_succeeds() {
        assertTrue(SpecialRollCalc.isAlwaysHungrySuccessful(2));
    }

    @Test
    void escapeFromAlwaysHungry_2plus_succeeds() {
        assertFalse(SpecialRollCalc.isEscapeFromAlwaysHungrySuccessful(1));
        assertTrue(SpecialRollCalc.isEscapeFromAlwaysHungrySuccessful(2));
        assertTrue(SpecialRollCalc.isEscapeFromAlwaysHungrySuccessful(6));
    }

    @Test
    void exhausted_only_on_1() {
        assertTrue(SpecialRollCalc.isExhausted(1));
        assertFalse(SpecialRollCalc.isExhausted(2));
    }

    // ── bribery / post-match ──────────────────────────────────────────────────

    @Test
    void bribes_2plus_succeed() {
        assertFalse(SpecialRollCalc.isBribesSuccessful(1));
        assertTrue(SpecialRollCalc.isBribesSuccessful(2));
        assertTrue(SpecialRollCalc.isBribesSuccessful(6));
    }

    @Test
    void argueCall_only_6_overturns() {
        assertFalse(SpecialRollCalc.isArgueTheCallSuccessful(5));
        assertTrue(SpecialRollCalc.isArgueTheCallSuccessful(6));
    }

    @Test
    void coachBanned_only_on_1() {
        assertTrue(SpecialRollCalc.isCoachBanned(1));
        assertFalse(SpecialRollCalc.isCoachBanned(2));
    }

    @Test
    void standUp_1_always_fails() {
        assertFalse(SpecialRollCalc.isStandUpSuccessful(1, 0));
    }

    @Test
    void standUp_4plus_succeeds() {
        assertFalse(SpecialRollCalc.isStandUpSuccessful(3, 0));   // 3 not > 3
        assertTrue(SpecialRollCalc.isStandUpSuccessful(4, 0));    // 4 > 3
    }

    @Test
    void standUp_modifier_adjusts_threshold() {
        assertFalse(SpecialRollCalc.isStandUpSuccessful(3, -1));  // 3-1=2 not > 3
        assertTrue(SpecialRollCalc.isStandUpSuccessful(3, 1));    // 3+1=4 > 3
    }

    @Test
    void playerDefecting_1to3() {
        assertTrue(SpecialRollCalc.isPlayerDefecting(1));
        assertTrue(SpecialRollCalc.isPlayerDefecting(3));
        assertFalse(SpecialRollCalc.isPlayerDefecting(4));
        assertFalse(SpecialRollCalc.isPlayerDefecting(0));
    }

    // ── kickoff events ────────────────────────────────────────────────────────

    @Test
    void riotRoll_below4_advancesTurn() {
        assertEquals(1, SpecialRollCalc.interpretRiotRoll(1));
        assertEquals(1, SpecialRollCalc.interpretRiotRoll(3));
    }

    @Test
    void riotRoll_4plus_goesBack() {
        assertEquals(-1, SpecialRollCalc.interpretRiotRoll(4));
        assertEquals(-1, SpecialRollCalc.interpretRiotRoll(6));
    }

    @Test
    void isDouble_sameValues() {
        assertTrue(SpecialRollCalc.isDouble(3, 3));
        assertFalse(SpecialRollCalc.isDouble(2, 3));
    }
}
