package com.fumbbl.ffb.server.util;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class KickoffEventCalcTest {

    // ── cheeringFansTotal ─────────────────────────────────────────────────────

    @Test
    void cheeringFans_roll4_fame3_cheerleaders2_is9() {
        assertEquals(9, KickoffEventCalc.cheeringFansTotal(4, 3, 2));
    }

    @Test
    void cheeringFans_no_cheerleaders() {
        assertEquals(7, KickoffEventCalc.cheeringFansTotal(4, 3, 0));
    }

    // ── brilliantCoachingTotal ────────────────────────────────────────────────

    @Test
    void brilliantCoaching_normalCoach() {
        assertEquals(9, KickoffEventCalc.brilliantCoachingTotal(4, 3, 2, false));
    }

    @Test
    void brilliantCoaching_bannedCoach_minus1() {
        assertEquals(8, KickoffEventCalc.brilliantCoachingTotal(4, 3, 2, true));
    }

    @Test
    void brilliantCoaching_noAssistants() {
        assertEquals(7, KickoffEventCalc.brilliantCoachingTotal(4, 3, 0, false));
    }

    // ── gainsExtraReroll ──────────────────────────────────────────────────────

    @Test
    void home_wins_higher_total() {
        assertTrue(KickoffEventCalc.gainsExtraReroll(8, 5));
    }

    @Test
    void away_wins_home_does_not() {
        assertFalse(KickoffEventCalc.gainsExtraReroll(5, 8));
    }

    @Test
    void tie_both_gain_reroll() {
        assertTrue(KickoffEventCalc.gainsExtraReroll(7, 7));
    }

    // ── combined scenario ─────────────────────────────────────────────────────

    @Test
    void scenario_cheering_fans_home_wins() {
        int homeTotal = KickoffEventCalc.cheeringFansTotal(4, 5, 2);  // 11
        int awayTotal = KickoffEventCalc.cheeringFansTotal(3, 4, 1);  // 8
        assertTrue(KickoffEventCalc.gainsExtraReroll(homeTotal, awayTotal));
        assertFalse(KickoffEventCalc.gainsExtraReroll(awayTotal, homeTotal));
    }

    @Test
    void scenario_brilliant_coaching_tie_both_win() {
        int homeTotal = KickoffEventCalc.brilliantCoachingTotal(3, 5, 2, false);  // 10
        int awayTotal = KickoffEventCalc.brilliantCoachingTotal(4, 4, 2, false);  // 10
        assertTrue(KickoffEventCalc.gainsExtraReroll(homeTotal, awayTotal));
        assertTrue(KickoffEventCalc.gainsExtraReroll(awayTotal, homeTotal));
    }
}
