package com.fumbbl.ffb.server.util;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.CsvSource;

import static org.junit.jupiter.api.Assertions.*;

class MovementCalcTest {

    // ── maxMovement ───────────────────────────────────────────────────────────

    @Test
    void maxMovement_noGfi_equalsMA() {
        assertEquals(6, MovementCalc.maxMovement(6, 0));
        assertEquals(4, MovementCalc.maxMovement(4, 0));
    }

    @Test
    void maxMovement_standardGfi_maPlus2() {
        assertEquals(8, MovementCalc.maxMovement(6, MovementCalc.STANDARD_GFI_SQUARES));
        assertEquals(6, MovementCalc.maxMovement(4, MovementCalc.STANDARD_GFI_SQUARES));
    }

    @Test
    void maxMovement_extraGfi_maPlus3() {
        assertEquals(9, MovementCalc.maxMovement(6, 3));
    }

    @Test
    void maxMovement_withTemporaryModifier_appliedToMA() {
        // MA 6, temporary +1 from skill → MA is passed as 7
        assertEquals(9, MovementCalc.maxMovement(7, MovementCalc.STANDARD_GFI_SQUARES));
    }

    // ── isNextMoveGoingForIt ──────────────────────────────────────────────────

    @ParameterizedTest(name = "currentMove={0} ma={1} → GFI={2}")
    @CsvSource({
        "0,6,false",
        "5,6,false",
        "6,6,true",   // exactly at MA → next is GFI
        "7,6,true",   // beyond MA → still GFI
        "0,4,false",
        "3,4,false",
        "4,4,true",
        "5,4,true",
    })
    void isNextMoveGoingForIt_variousCases(int currentMove, int ma, boolean expected) {
        assertEquals(expected, MovementCalc.isNextMoveGoingForIt(currentMove, ma));
    }

    @Test
    void isNextMoveGoingForIt_ma1_gfiImmediatelyAfterFirstMove() {
        // Snotling with MA 1: first GFI after move 1
        assertFalse(MovementCalc.isNextMoveGoingForIt(0, 1));
        assertTrue(MovementCalc.isNextMoveGoingForIt(1, 1));
    }

    // ── mustRollToStandUp ─────────────────────────────────────────────────────

    @ParameterizedTest(name = "ma={0} → mustRoll={1}")
    @CsvSource({
        "1,true",
        "2,true",
        "3,true",
        "4,false",
        "5,false",
        "6,false",
        "9,false",
    })
    void mustRollToStandUp_ma3OrUnder_requiresRoll(int ma, boolean expected) {
        assertEquals(expected, MovementCalc.mustRollToStandUp(ma));
    }

    @Test
    void mustRollToStandUp_boundary_exactly3_mustRoll() {
        assertTrue(MovementCalc.mustRollToStandUp(3));
    }

    @Test
    void mustRollToStandUp_boundary_4_noRoll() {
        assertFalse(MovementCalc.mustRollToStandUp(4));
    }

    // ── hasMoveLeft ───────────────────────────────────────────────────────────

    @Test
    void hasMoveLeft_notYetMoved_alwaysHasMove() {
        assertTrue(MovementCalc.hasMoveLeft(0, 6, 0));
    }

    @Test
    void hasMoveLeft_reachedMAexact_noMoveWithoutGfi() {
        assertFalse(MovementCalc.hasMoveLeft(6, 6, 0));
    }

    @Test
    void hasMoveLeft_reachedMA_hasGfiSquaresLeft() {
        assertTrue(MovementCalc.hasMoveLeft(6, 6, MovementCalc.STANDARD_GFI_SQUARES));
    }

    @Test
    void hasMoveLeft_usedAllGfi_noMoveLeft() {
        // MA 6 + 2 GFI = 8 total; after moving 8 → no move left
        assertFalse(MovementCalc.hasMoveLeft(8, 6, MovementCalc.STANDARD_GFI_SQUARES));
    }

    // ── constants ─────────────────────────────────────────────────────────────

    @Test
    void constants_valuesAreCorrect() {
        assertEquals(2, MovementCalc.STANDARD_GFI_SQUARES);
        assertEquals(3, MovementCalc.STAND_UP_COST);
        assertEquals(2, MovementCalc.GFI_MINIMUM_ROLL);
    }

    @Test
    void gfiSquares_noSkill_returns2() {
        assertEquals(2, MovementCalc.gfiSquares(false));
    }

    @Test
    void gfiSquares_withExtraGfi_returns3() {
        assertEquals(3, MovementCalc.gfiSquares(true));
    }

    // ── interaction: GFI minimum roll ────────────────────────────────────────

    @Test
    void gfiMinimumRoll_isAlways2_regardlessOfStats() {
        // GFI always requires 2+ (not agility-based), same across all editions
        assertEquals(2, MovementCalc.GFI_MINIMUM_ROLL);
    }
}
