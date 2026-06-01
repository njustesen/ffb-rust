package com.fumbbl.ffb.server.util;

import com.fumbbl.ffb.PassingDistance;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.CsvSource;

import static org.junit.jupiter.api.Assertions.*;

class PassingDistanceCalcTest {

    // ── Own square ────────────────────────────────────────────────────────────

    @Test
    void sameSquare_returnsNull() {
        assertNull(PassingDistanceCalc.forDeltas(0, 0));
    }

    // ── Quick Pass ────────────────────────────────────────────────────────────

    @ParameterizedTest(name = "dx={0}, dy={1}")
    @CsvSource({
        "1,0",  // adjacent horizontally
        "0,1",  // adjacent vertically
        "1,1",  // diagonal
        "2,0",  // two squares horizontal
        "2,1",  // row 1
        "2,2",  // row 2
        "1,2",  // row 2
        "3,0",  // row 0
        "3,1",  // row 1
    })
    void quickPass(int dx, int dy) {
        assertEquals(PassingDistance.QUICK_PASS, PassingDistanceCalc.forDeltas(dx, dy));
    }

    // ── Short Pass ────────────────────────────────────────────────────────────

    @ParameterizedTest(name = "dx={0}, dy={1}")
    @CsvSource({
        "4,0",   // row 0
        "5,0",   // row 0
        "6,0",   // row 0
        "3,2",   // row 2
        "4,2",   // row 2
        "0,4",   // dy=4
        "1,4",   // row 4
    })
    void shortPass(int dx, int dy) {
        assertEquals(PassingDistance.SHORT_PASS, PassingDistanceCalc.forDeltas(dx, dy));
    }

    // ── Long Pass ─────────────────────────────────────────────────────────────

    @ParameterizedTest(name = "dx={0}, dy={1}")
    @CsvSource({
        "7,0",   // row 0
        "8,0",   // row 0
        "9,0",   // row 0
        "10,0",  // row 0
        "0,7",   // dy=7
        "1,7",   // row 7
    })
    void longPass(int dx, int dy) {
        assertEquals(PassingDistance.LONG_PASS, PassingDistanceCalc.forDeltas(dx, dy));
    }

    // ── Long Bomb ─────────────────────────────────────────────────────────────

    @ParameterizedTest(name = "dx={0}, dy={1}")
    @CsvSource({
        "11,0",  // row 0
        "12,0",  // row 0
        "13,0",  // row 0
        "0,11",  // dy=11
        "1,11",  // row 11
        "0,12",  // dy=12
        "0,13",  // dy=13
        "1,12",  // row 12
        "2,13",  // row 13
    })
    void longBomb(int dx, int dy) {
        assertEquals(PassingDistance.LONG_BOMB, PassingDistanceCalc.forDeltas(dx, dy));
    }

    // ── Out of range ──────────────────────────────────────────────────────────

    @Test
    void negativeDelta_returnsNull() {
        assertNull(PassingDistanceCalc.forDeltas(-1, 0));
        assertNull(PassingDistanceCalc.forDeltas(0, -1));
    }

    @Test
    void deltaGreaterThan13_returnsNull() {
        assertNull(PassingDistanceCalc.forDeltas(14, 0));
        assertNull(PassingDistanceCalc.forDeltas(0, 14));
    }

    // ── Null cells in table (spaces) ──────────────────────────────────────────

    @Test
    void outOfRangeCells_returnNull() {
        // dy=13, dx=3 → "B B B   " → index 3 is space → null
        assertNull(PassingDistanceCalc.forDeltas(3, 13));
    }

    // ── forCoordinates ────────────────────────────────────────────────────────

    @Test
    void forCoordinates_symmetrical() {
        // Passing from (5,7) to (8,7) = dx=3, dy=0 → QuickPass
        assertEquals(PassingDistance.QUICK_PASS, PassingDistanceCalc.forCoordinates(5, 7, 8, 7));
        // Symmetric: (8,7) to (5,7)
        assertEquals(PassingDistance.QUICK_PASS, PassingDistanceCalc.forCoordinates(8, 7, 5, 7));
    }

    @Test
    void forCoordinates_sameSquare_returnsNull() {
        assertNull(PassingDistanceCalc.forCoordinates(5, 7, 5, 7));
    }

    @Test
    void forCoordinates_longBomb_acrossField() {
        // From x=1 to x=14 = dx=13, dy=0 → LongBomb
        assertEquals(PassingDistance.LONG_BOMB, PassingDistanceCalc.forCoordinates(1, 7, 14, 7));
    }
}
