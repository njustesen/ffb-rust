package com.fumbbl.ffb.server.util;

import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.RulesCollection.Rules;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.CsvSource;

import static org.junit.jupiter.api.Assertions.*;

class ThrowInCalcTest {

    // ── throwInDistance ───────────────────────────────────────────────────────

    @Test
    void distance_bb2016_sumsTwoDice() {
        assertEquals(7, ThrowInCalc.throwInDistance(3, 4, Rules.BB2016));
        assertEquals(2, ThrowInCalc.throwInDistance(1, 1, Rules.BB2016));
        assertEquals(12, ThrowInCalc.throwInDistance(6, 6, Rules.BB2016));
    }

    @Test
    void distance_bb2020_addsBonusOne() {
        assertEquals(8, ThrowInCalc.throwInDistance(3, 4, Rules.BB2020));
        assertEquals(3, ThrowInCalc.throwInDistance(1, 1, Rules.BB2020));
        assertEquals(13, ThrowInCalc.throwInDistance(6, 6, Rules.BB2020));
    }

    @Test
    void distance_bb2025_sumsTwoDice_noBonusLikeBb2016() {
        assertEquals(7, ThrowInCalc.throwInDistance(3, 4, Rules.BB2025));
        assertEquals(2, ThrowInCalc.throwInDistance(1, 1, Rules.BB2025));
    }

    // ── isCornerSquare ────────────────────────────────────────────────────────

    @Test
    void isCornerSquare_allFourCorners() {
        assertTrue(ThrowInCalc.isCornerSquare(0, 0));
        assertTrue(ThrowInCalc.isCornerSquare(25, 0));
        assertTrue(ThrowInCalc.isCornerSquare(0, 14));
        assertTrue(ThrowInCalc.isCornerSquare(25, 14));
    }

    @Test
    void isCornerSquare_edgeNotCorner() {
        assertFalse(ThrowInCalc.isCornerSquare(5, 0));   // upper sideline, not corner
        assertFalse(ThrowInCalc.isCornerSquare(0, 7));   // home endzone, not corner
        assertFalse(ThrowInCalc.isCornerSquare(12, 7));  // field
    }

    // ── throwInDirectionForRoll ───────────────────────────────────────────────

    @ParameterizedTest(name = "home-endzone roll {0} → {1}")
    @CsvSource({ "1,NORTHEAST", "2,NORTHEAST", "3,EAST", "4,EAST", "5,SOUTHEAST", "6,SOUTHEAST" })
    void homeEndzone_directionsForRolls(int roll, Direction expected) {
        assertEquals(expected, ThrowInCalc.throwInDirectionForRoll(0, 7, roll));
    }

    @ParameterizedTest(name = "away-endzone roll {0} → {1}")
    @CsvSource({ "1,SOUTHWEST", "2,SOUTHWEST", "3,WEST", "4,WEST", "5,NORTHWEST", "6,NORTHWEST" })
    void awayEndzone_directionsForRolls(int roll, Direction expected) {
        assertEquals(expected, ThrowInCalc.throwInDirectionForRoll(25, 7, roll));
    }

    @ParameterizedTest(name = "lower-sideline roll {0} → {1}")
    @CsvSource({ "1,NORTHWEST", "2,NORTHWEST", "3,NORTH", "4,NORTH", "5,NORTHEAST", "6,NORTHEAST" })
    void lowerSideline_directionsForRolls(int roll, Direction expected) {
        assertEquals(expected, ThrowInCalc.throwInDirectionForRoll(12, 14, roll));
    }

    @ParameterizedTest(name = "upper-sideline roll {0} → {1}")
    @CsvSource({ "1,SOUTHEAST", "2,SOUTHEAST", "3,SOUTH", "4,SOUTH", "5,SOUTHWEST", "6,SOUTHWEST" })
    void upperSideline_directionsForRolls(int roll, Direction expected) {
        assertEquals(expected, ThrowInCalc.throwInDirectionForRoll(12, 0, roll));
    }

    // ── cornerThrowInDirectionForRoll ─────────────────────────────────────────

    @ParameterizedTest(name = "NW-corner D3={0} → {1}")
    @CsvSource({ "1,EAST", "2,SOUTHEAST", "3,SOUTH" })
    void northwestCorner(int roll, Direction expected) {
        assertEquals(expected, ThrowInCalc.cornerThrowInDirectionForRoll(Direction.NORTHWEST, roll));
    }

    @ParameterizedTest(name = "NE-corner D3={0} → {1}")
    @CsvSource({ "1,SOUTH", "2,SOUTHWEST", "3,WEST" })
    void northeastCorner(int roll, Direction expected) {
        assertEquals(expected, ThrowInCalc.cornerThrowInDirectionForRoll(Direction.NORTHEAST, roll));
    }

    @ParameterizedTest(name = "SW-corner D3={0} → {1}")
    @CsvSource({ "1,NORTH", "2,NORTHEAST", "3,EAST" })
    void southwestCorner(int roll, Direction expected) {
        assertEquals(expected, ThrowInCalc.cornerThrowInDirectionForRoll(Direction.SOUTHWEST, roll));
    }

    @ParameterizedTest(name = "SE-corner D3={0} → {1}")
    @CsvSource({ "1,WEST", "2,NORTHWEST", "3,NORTH" })
    void southeastCorner(int roll, Direction expected) {
        assertEquals(expected, ThrowInCalc.cornerThrowInDirectionForRoll(Direction.SOUTHEAST, roll));
    }

    // ── cornerDirection ───────────────────────────────────────────────────────

    @Test
    void cornerDirection_allFourCorners() {
        assertEquals(Direction.NORTHWEST, ThrowInCalc.cornerDirection(0, 0));
        assertEquals(Direction.NORTHEAST, ThrowInCalc.cornerDirection(25, 0));
        assertEquals(Direction.SOUTHWEST, ThrowInCalc.cornerDirection(0, 14));
        assertEquals(Direction.SOUTHEAST, ThrowInCalc.cornerDirection(25, 14));
    }
}
