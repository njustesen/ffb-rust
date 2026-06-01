package com.fumbbl.ffb.server.util;

import com.fumbbl.ffb.PassingDistance;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.CsvSource;

import static org.junit.jupiter.api.Assertions.*;

class PassCalcTest {

    // ── minimumRollPassBB2016 ─────────────────────────────────────────────────

    @Test
    void bb2016_shortPass_ag3() {
        // AG3, short pass (dist_mod=0): 7 - 3 - 0 = 4
        assertEquals(4, PassCalc.minimumRollPassBB2016(3, PassingDistance.SHORT_PASS, 0));
    }

    @Test
    void bb2016_longPass_ag4() {
        // AG4, long pass (dist_mod=-1): 7 - 4 + 1 = 4
        assertEquals(4, PassCalc.minimumRollPassBB2016(4, PassingDistance.LONG_PASS, 0));
    }

    @Test
    void bb2016_quickPass_ag6_floorsAt2() {
        // AG6, quick pass (dist_mod=+1): 7 - 6 - 1 = 0 → 2; fumble_boundary = 2 - 1 = 1 → max = 2
        assertEquals(2, PassCalc.minimumRollPassBB2016(6, PassingDistance.QUICK_PASS, 0));
    }

    @Test
    void bb2016_longBomb_ag6_fumbleboundaryDominates() {
        // AG6, long bomb (dist_mod=-2): ag_based = 7-6+2 = 3; fumble_boundary = 2+2 = 4
        // max(3, 4, 2) = 4 — fumble boundary dominates, not the agility formula
        assertEquals(4, PassCalc.minimumRollPassBB2016(6, PassingDistance.LONG_BOMB, 0));
    }

    @Test
    void bb2016_longBomb_ag4() {
        // AG4, long bomb: ag_based = 7-4+2 = 5; fumble_boundary = 4; max = 5
        assertEquals(5, PassCalc.minimumRollPassBB2016(4, PassingDistance.LONG_BOMB, 0));
    }

    @Test
    void bb2016_withPositiveModifier_harder() {
        // AG4, short pass, +1 modifier (e.g. rain): 7-4-0+1 = 4; fumble_boundary=2+1=3; max=4
        assertEquals(4, PassCalc.minimumRollPassBB2016(4, PassingDistance.SHORT_PASS, 1));
    }

    @Test
    void bb2016_withNegativeModifier_easier_floorAt2() {
        // AG4, short pass, -2 (e.g. strong arm): 7-4-0-2 = 1 → 2; fumble=2-0-2=0 → max = 2
        assertEquals(2, PassCalc.minimumRollPassBB2016(4, PassingDistance.SHORT_PASS, -2));
    }

    @ParameterizedTest(name = "distance={0} → target={1}")
    @CsvSource({
        "QUICK_PASS,2",    // 7-4-1=2, fumble=2-1=1; max=2
        "SHORT_PASS,3",    // 7-4-0=3, fumble=2; max=3
        "LONG_PASS,4",     // 7-4+1=4, fumble=3; max=4
        "LONG_BOMB,5",     // 7-4+2=5, fumble=4; max=5
    })
    void bb2016_ag4_allDistances(PassingDistance distance, int expected) {
        assertEquals(expected, PassCalc.minimumRollPassBB2016(4, distance, 0));
    }

    // ── minimumRollPassBB2020 ─────────────────────────────────────────────────

    @Test
    void bb2020_noPa_returnsNull() {
        assertNull(PassCalc.minimumRollPassBB2020(0, PassingDistance.SHORT_PASS, 0));
    }

    @Test
    void bb2020_pa2_shortPass() {
        // PA2 + dist_mod 1 = 3
        assertEquals(3, PassCalc.minimumRollPassBB2020(2, PassingDistance.SHORT_PASS, 0));
    }

    @Test
    void bb2020_pa3_quickPass() {
        // PA3 + dist_mod 0 = 3
        assertEquals(3, PassCalc.minimumRollPassBB2020(3, PassingDistance.QUICK_PASS, 0));
    }

    @Test
    void bb2020_pa3_longBomb() {
        // PA3 + dist_mod 3 = 6
        assertEquals(6, PassCalc.minimumRollPassBB2020(3, PassingDistance.LONG_BOMB, 0));
    }

    @Test
    void bb2020_withModifier() {
        // PA3, short pass (1), +1 rain = 5
        assertEquals(5, PassCalc.minimumRollPassBB2020(3, PassingDistance.SHORT_PASS, 1));
    }

    @Test
    void bb2020_floorAt2() {
        // PA2 + dist_mod 0 - 5 modifiers = -3 → 2
        assertEquals(2, PassCalc.minimumRollPassBB2020(2, PassingDistance.QUICK_PASS, -5));
    }

    // ── isModifiedFumbleBB2016 ───────────────────────────────────────────────

    @Test
    void bb2016_naturalOne_quickPass_notModifiedFumble_handleSeparately() {
        // Natural 1 fumble is handled separately as a direct rule, not via isModifiedFumble.
        // roll=1, quick pass dist_mod=+1: 1+1-0=2 > 1 → NOT a modified fumble
        // (caller must check roll==1 as a direct fumble first)
        assertFalse(PassCalc.isModifiedFumbleBB2016(1, PassingDistance.QUICK_PASS, 0));
    }

    @Test
    void bb2016_longBomb_roll2_isFumble() {
        // roll 2 + dist_mod(-2) - 0 = 0 <= 1 → fumble
        assertTrue(PassCalc.isModifiedFumbleBB2016(2, PassingDistance.LONG_BOMB, 0));
    }

    @Test
    void bb2016_shortPass_roll2_notFumble() {
        // roll 2 + dist_mod(0) - 0 = 2 > 1 → not fumble
        assertFalse(PassCalc.isModifiedFumbleBB2016(2, PassingDistance.SHORT_PASS, 0));
    }

    @Test
    void bb2016_quickPass_roll1_notModifiedFumble() {
        // roll 1 + dist_mod(+1) - 0 = 2 > 1 → not a modified fumble
        // (natural 1 is handled separately as a direct fumble regardless)
        assertFalse(PassCalc.isModifiedFumbleBB2016(1, PassingDistance.QUICK_PASS, 0));
    }
}
