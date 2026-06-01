package com.fumbbl.ffb.server.util;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.CsvSource;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.junit.jupiter.api.Assertions.assertEquals;

class FoulCalcTest {

    // ── isSpottedByArmorRoll ──────────────────────────────────────────────────

    @Test
    void armorRoll_doublesNoSneakyGit_spotted() {
        assertTrue(FoulCalc.isSpottedByArmorRoll(3, 3, false));
    }

    @Test
    void armorRoll_doublesWithSneakyGit_notSpotted() {
        assertFalse(FoulCalc.isSpottedByArmorRoll(3, 3, true));
    }

    @Test
    void armorRoll_nonDoubles_notSpotted() {
        assertFalse(FoulCalc.isSpottedByArmorRoll(2, 4, false));
    }

    @ParameterizedTest
    @CsvSource({"1,1", "2,2", "3,3", "4,4", "5,5", "6,6"})
    void armorRoll_allDoublesNoSneakyGit_spotted(int d1, int d2) {
        assertTrue(FoulCalc.isSpottedByArmorRoll(d1, d2, false));
    }

    @ParameterizedTest
    @CsvSource({"1,1", "2,2", "3,3", "4,4", "5,5", "6,6"})
    void armorRoll_allDoublesSneakyGit_notSpotted(int d1, int d2) {
        assertFalse(FoulCalc.isSpottedByArmorRoll(d1, d2, true));
    }

    // ── isSpottedByInjuryRoll ─────────────────────────────────────────────────

    @Test
    void injuryRoll_doublesArmorBroken_spotted() {
        assertTrue(FoulCalc.isSpottedByInjuryRoll(4, 4, true));
    }

    @Test
    void injuryRoll_doublesArmorNotBroken_notSpotted() {
        assertFalse(FoulCalc.isSpottedByInjuryRoll(4, 4, false));
    }

    @Test
    void injuryRoll_nonDoublesArmorBroken_notSpotted() {
        assertFalse(FoulCalc.isSpottedByInjuryRoll(3, 5, true));
    }

    @ParameterizedTest
    @CsvSource({"1,1", "2,2", "3,3", "4,4", "5,5", "6,6"})
    void injuryRoll_allDoublesWhenArmorBroken_spotted(int d1, int d2) {
        assertTrue(FoulCalc.isSpottedByInjuryRoll(d1, d2, true));
    }

    // ── isSpottedByReferee (combined) ─────────────────────────────────────────

    @Test
    void referee_armorDoubles_spotted() {
        // Armor doubles, armor not broken, no SneakyGit
        assertTrue(FoulCalc.isSpottedByReferee(2, 2, 1, 3, false, false));
    }

    @Test
    void referee_injuryDoublesArmorBroken_spotted() {
        // Non-double armor, armor broken, injury doubles
        assertTrue(FoulCalc.isSpottedByReferee(3, 5, 4, 4, true, false));
    }

    @Test
    void referee_noDoubles_notSpotted() {
        // No doubles anywhere
        assertFalse(FoulCalc.isSpottedByReferee(2, 4, 3, 5, true, false));
    }

    @Test
    void referee_armorDoublesSneakyGit_injuryNotDoubles_notSpotted() {
        // SneakyGit suppresses armor-roll detection, injury not doubles
        assertFalse(FoulCalc.isSpottedByReferee(3, 3, 2, 5, false, true));
    }

    @Test
    void referee_armorDoublesSneakyGit_injuryDoublesArmorBroken_spotted() {
        // SneakyGit suppresses armor-roll detection, but injury doubles fires
        assertTrue(FoulCalc.isSpottedByReferee(3, 3, 4, 4, true, true));
    }

    @Test
    void referee_noArmorRoll_injuryDoublesArmorNotBroken_notSpotted() {
        // Armor not broken (no injury roll made), doubles on injury ignored
        assertFalse(FoulCalc.isSpottedByReferee(2, 4, 3, 3, false, false));
    }

    // ── minimumRollToBreakArmour ──────────────────────────────────────────────

    @ParameterizedTest
    @CsvSource({"7,8", "8,9", "9,10", "10,11", "11,12"})
    void minimumRollToBreakArmour_avPlusOne(int av, int expected) {
        assertEquals(expected, FoulCalc.minimumRollToBreakArmour(av));
    }
}
