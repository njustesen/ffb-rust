package com.fumbbl.ffb.server.mechanic;

import com.fumbbl.ffb.RulesCollection.Rules;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class SppCalcTest {

    // ── Touchdown ─────────────────────────────────────────────────────────────

    @Test
    void touchdown_bb2016_is3() {
        assertEquals(3, SppCalc.touchdownSpp(Rules.BB2016));
    }

    @Test
    void touchdown_bb2020_is3() {
        assertEquals(3, SppCalc.touchdownSpp(Rules.BB2020));
    }

    @Test
    void touchdown_bb2025_normalTeam_is3() {
        assertEquals(3, SppCalc.touchdownSpp(Rules.BB2025, false));
    }

    @Test
    void touchdown_bb2025_brawlinBrutes_is2() {
        assertEquals(2, SppCalc.touchdownSpp(Rules.BB2025, true));
    }

    @Test
    void touchdown_bb2016_brawlinBrutesHasNoEffect() {
        // Brawlin' Brutes rule only exists in BB2025
        assertEquals(3, SppCalc.touchdownSpp(Rules.BB2016, true));
    }

    // ── Casualty ─────────────────────────────────────────────────────────────

    @Test
    void casualty_bb2016_is2() {
        assertEquals(2, SppCalc.casualtySpp(Rules.BB2016));
    }

    @Test
    void casualty_bb2020_is2() {
        assertEquals(2, SppCalc.casualtySpp(Rules.BB2020));
    }

    @Test
    void casualty_bb2025_normalTeam_is2() {
        assertEquals(2, SppCalc.casualtySpp(Rules.BB2025, false));
    }

    @Test
    void casualty_bb2025_brawlinBrutes_is3() {
        assertEquals(3, SppCalc.casualtySpp(Rules.BB2025, true));
    }

    // ── Constant events (same across all editions) ────────────────────────────

    @Test
    void completion_is1_allEditions() {
        assertEquals(1, SppCalc.completionSpp());
    }

    @Test
    void interception_is2_allEditions() {
        assertEquals(2, SppCalc.interceptionSpp());
    }

    @Test
    void deflection_is1_allEditions() {
        assertEquals(1, SppCalc.deflectionSpp());
    }

    @Test
    void catch_is1_allEditions() {
        assertEquals(1, SppCalc.catchSpp());
    }

    // ── Landing ──────────────────────────────────────────────────────────────

    @Test
    void landing_bb2016_is0() {
        assertEquals(0, SppCalc.landingSpp(Rules.BB2016));
    }

    @Test
    void landing_bb2020_is0() {
        assertEquals(0, SppCalc.landingSpp(Rules.BB2020));
    }

    @Test
    void landing_bb2025_is1() {
        assertEquals(1, SppCalc.landingSpp(Rules.BB2025));
    }

    // ── MVP ──────────────────────────────────────────────────────────────────

    @Test
    void mvp_bb2016_is5() {
        assertEquals(5, SppCalc.mvpSpp(Rules.BB2016));
    }

    @Test
    void mvp_bb2020_is4() {
        assertEquals(4, SppCalc.mvpSpp(Rules.BB2020));
    }

    @Test
    void mvp_bb2025_is4() {
        assertEquals(4, SppCalc.mvpSpp(Rules.BB2025));
    }

    // ── Additional SPP (league bonus) ─────────────────────────────────────────

    @Test
    void additionalSpp_bb2016_is0() {
        assertEquals(0, SppCalc.additionalSpp(Rules.BB2016));
    }

    @Test
    void additionalSpp_bb2020_is1() {
        assertEquals(1, SppCalc.additionalSpp(Rules.BB2020));
    }

    @Test
    void additionalSpp_bb2025_is1() {
        assertEquals(1, SppCalc.additionalSpp(Rules.BB2025));
    }

    // ── Level thresholds (BB2016 SPP-based) ──────────────────────────────────

    @Test
    void bb2016_level_thresholds_values() {
        assertArrayEquals(new int[]{6, 16, 31, 51, 76, 176}, SppCalc.LEVEL_THRESHOLDS_BB2016);
    }

    @Test
    void bb2016_playerLevel_rookie() {
        assertEquals(0, SppCalc.playerLevelBB2016(0));
        assertEquals(0, SppCalc.playerLevelBB2016(5));
    }

    @Test
    void bb2016_playerLevel_experienced() {
        assertEquals(1, SppCalc.playerLevelBB2016(6));
        assertEquals(1, SppCalc.playerLevelBB2016(15));
    }

    @Test
    void bb2016_playerLevel_veteran() {
        assertEquals(2, SppCalc.playerLevelBB2016(16));
        assertEquals(2, SppCalc.playerLevelBB2016(30));
    }

    @Test
    void bb2016_playerLevel_emerging() {
        assertEquals(3, SppCalc.playerLevelBB2016(31));
        assertEquals(3, SppCalc.playerLevelBB2016(50));
    }

    @Test
    void bb2016_playerLevel_star() {
        assertEquals(4, SppCalc.playerLevelBB2016(51));
        assertEquals(4, SppCalc.playerLevelBB2016(75));
    }

    @Test
    void bb2016_playerLevel_superStar() {
        assertEquals(5, SppCalc.playerLevelBB2016(76));
        assertEquals(5, SppCalc.playerLevelBB2016(175));
    }

    @Test
    void bb2016_playerLevel_legend() {
        assertEquals(6, SppCalc.playerLevelBB2016(176));
        assertEquals(6, SppCalc.playerLevelBB2016(999));
    }

    @Test
    void bb2016_justLevelledUp_atThreshold() {
        assertTrue(SppCalc.justLevelledUpBB2016(5, 6));    // 0→1
        assertTrue(SppCalc.justLevelledUpBB2016(15, 16));  // 1→2
        assertTrue(SppCalc.justLevelledUpBB2016(30, 31));  // 2→3
        assertFalse(SppCalc.justLevelledUpBB2016(6, 15));  // same level
        assertFalse(SppCalc.justLevelledUpBB2016(16, 30)); // same level
    }
}
