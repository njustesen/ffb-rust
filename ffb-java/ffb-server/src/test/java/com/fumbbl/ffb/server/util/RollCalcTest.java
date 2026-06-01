package com.fumbbl.ffb.server.util;

import com.fumbbl.ffb.RulesCollection.Rules;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.CsvSource;

import static org.junit.jupiter.api.Assertions.*;

class RollCalcTest {

    // ── isSkillRollSuccessful ────────────────────────────────────────────────

    @Test
    void naturalSix_alwaysSucceeds_regardlessOfMinimum() {
        assertTrue(RollCalc.isSkillRollSuccessful(6, 7)); // even with impossible target
        assertTrue(RollCalc.isSkillRollSuccessful(6, 6));
        assertTrue(RollCalc.isSkillRollSuccessful(6, 2));
    }

    @Test
    void naturalOne_alwaysFails_regardlessOfMinimum() {
        assertFalse(RollCalc.isSkillRollSuccessful(1, 1)); // even with minimum 1
        assertFalse(RollCalc.isSkillRollSuccessful(1, 2));
        assertFalse(RollCalc.isSkillRollSuccessful(1, 6));
    }

    @ParameterizedTest(name = "roll={0} min={1} → success={2}")
    @CsvSource({
        "2,2,true",
        "3,2,true",
        "5,4,true",
        "4,5,false",
        "3,4,false",
        "2,3,false",
    })
    void normalRoll_meetsOrBeatsMinimum(int roll, int min, boolean expected) {
        assertEquals(expected, RollCalc.isSkillRollSuccessful(roll, min));
    }

    // ── isArmourBroken ───────────────────────────────────────────────────────

    @ParameterizedTest(name = "BB2016 armour={0} rollTotal={1} → broken={2}")
    @CsvSource({
        "7,7,false",   // equal: NOT broken in BB2016 (strict >)
        "7,8,true",    // one over: broken
        "9,9,false",   // equal: not broken
        "9,10,true",   // one over: broken
        "7,6,false",   // under: not broken
    })
    void bb2016_armourBroken_strictlyGreater(int armour, int rollTotal, boolean expected) {
        assertEquals(expected, RollCalc.isArmourBroken(armour, rollTotal, Rules.BB2016));
    }

    @ParameterizedTest(name = "BB2020 armour={0} rollTotal={1} → broken={2}")
    @CsvSource({
        "7,7,true",    // equal: broken in BB2020 (>=)
        "7,8,true",    // one over: broken
        "9,9,true",    // equal: broken
        "7,6,false",   // under: not broken
        "9,8,false",   // under: not broken
    })
    void bb2020_armourBroken_equalOrGreater(int armour, int rollTotal, boolean expected) {
        assertEquals(expected, RollCalc.isArmourBroken(armour, rollTotal, Rules.BB2020));
    }

    @Test
    void bb2025_sameAsBB2020() {
        // BB2025 uses the same >= comparison as BB2020
        assertEquals(RollCalc.isArmourBroken(7, 7, Rules.BB2020),
                     RollCalc.isArmourBroken(7, 7, Rules.BB2025));
        assertEquals(RollCalc.isArmourBroken(9, 9, Rules.BB2020),
                     RollCalc.isArmourBroken(9, 9, Rules.BB2025));
    }

    @Test
    void edition_difference_atEqualRoll_boundary() {
        // Exact boundary where editions differ: armour == rollTotal
        assertFalse(RollCalc.isArmourBroken(8, 8, Rules.BB2016)); // NOT broken
        assertTrue(RollCalc.isArmourBroken(8, 8, Rules.BB2020));  // broken
    }

    // ── applyFixedArmourReduction ────────────────────────────────────────────

    @Test
    void chainsaw_bb2016_capsAt7() {
        assertEquals(7, RollCalc.applyFixedArmourReduction(10, Rules.BB2016));
        assertEquals(7, RollCalc.applyFixedArmourReduction(8, Rules.BB2016));
        assertEquals(7, RollCalc.applyFixedArmourReduction(7, Rules.BB2016));
        assertEquals(6, RollCalc.applyFixedArmourReduction(6, Rules.BB2016)); // below cap, unchanged
    }

    @Test
    void chainsaw_bb2020_capsAt8() {
        assertEquals(8, RollCalc.applyFixedArmourReduction(10, Rules.BB2020));
        assertEquals(8, RollCalc.applyFixedArmourReduction(9, Rules.BB2020));
        assertEquals(8, RollCalc.applyFixedArmourReduction(8, Rules.BB2020));
        assertEquals(7, RollCalc.applyFixedArmourReduction(7, Rules.BB2020)); // below cap, unchanged
    }

    @Test
    void chainsaw_bb2025_sameAsBB2020() {
        assertEquals(RollCalc.applyFixedArmourReduction(10, Rules.BB2020),
                     RollCalc.applyFixedArmourReduction(10, Rules.BB2025));
    }

    // ── minimumRollGoingForIt ────────────────────────────────────────────────

    @Test
    void gfi_noModifier_requires2() {
        assertEquals(2, RollCalc.minimumRollGoingForIt(0));
    }

    @Test
    void gfi_positiveModifier_increases() {
        assertEquals(3, RollCalc.minimumRollGoingForIt(1));
        assertEquals(4, RollCalc.minimumRollGoingForIt(2));
    }

    @Test
    void gfi_negativeModifier_floorAt2() {
        // Negative modifiers cannot push below 2
        assertEquals(2, RollCalc.minimumRollGoingForIt(-1));
        assertEquals(2, RollCalc.minimumRollGoingForIt(-5));
    }

    @Test
    void gfi_minimum_is_always_2() {
        for (int mod = -10; mod <= 0; mod++) {
            assertEquals(2, RollCalc.minimumRollGoingForIt(mod),
                "modifier=" + mod + " should never push GFI below 2");
        }
    }
}
