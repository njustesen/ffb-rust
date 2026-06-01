package com.fumbbl.ffb.server.util;

import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.FieldCoordinate;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.CsvSource;

import static org.junit.jupiter.api.Assertions.*;

class ScatterCalcTest {

    // ── directionForRoll ──────────────────────────────────────────────────────

    @ParameterizedTest(name = "roll {0} → {1}")
    @CsvSource({
        "1, NORTH",
        "2, NORTHEAST",
        "3, EAST",
        "4, SOUTHEAST",
        "5, SOUTH",
        "6, SOUTHWEST",
        "7, WEST",
        "8, NORTHWEST"
    })
    void directionForRoll_allFaces(int roll, Direction expected) {
        assertEquals(expected, ScatterCalc.directionForRoll(roll));
    }

    @Test
    void directionForRoll_outOfRange_returnsNull() {
        assertNull(ScatterCalc.directionForRoll(0));
        assertNull(ScatterCalc.directionForRoll(9));
    }

    // ── scatterCoordinate ─────────────────────────────────────────────────────

    @Test
    void scatter_north_decreasesY() {
        FieldCoordinate result = ScatterCalc.scatterCoordinate(new FieldCoordinate(10, 10), Direction.NORTH, 1);
        assertEquals(new FieldCoordinate(10, 9), result);
    }

    @Test
    void scatter_south_increasesY() {
        FieldCoordinate result = ScatterCalc.scatterCoordinate(new FieldCoordinate(10, 10), Direction.SOUTH, 1);
        assertEquals(new FieldCoordinate(10, 11), result);
    }

    @Test
    void scatter_east_increasesX() {
        FieldCoordinate result = ScatterCalc.scatterCoordinate(new FieldCoordinate(10, 10), Direction.EAST, 1);
        assertEquals(new FieldCoordinate(11, 10), result);
    }

    @Test
    void scatter_west_decreasesX() {
        FieldCoordinate result = ScatterCalc.scatterCoordinate(new FieldCoordinate(10, 10), Direction.WEST, 1);
        assertEquals(new FieldCoordinate(9, 10), result);
    }

    @Test
    void scatter_northeast_increasesBoth() {
        FieldCoordinate result = ScatterCalc.scatterCoordinate(new FieldCoordinate(10, 10), Direction.NORTHEAST, 1);
        assertEquals(new FieldCoordinate(11, 9), result);
    }

    @Test
    void scatter_southeast_increasesXincreasesY() {
        FieldCoordinate result = ScatterCalc.scatterCoordinate(new FieldCoordinate(10, 10), Direction.SOUTHEAST, 1);
        assertEquals(new FieldCoordinate(11, 11), result);
    }

    @Test
    void scatter_southwest_decreasesXincreasesY() {
        FieldCoordinate result = ScatterCalc.scatterCoordinate(new FieldCoordinate(10, 10), Direction.SOUTHWEST, 1);
        assertEquals(new FieldCoordinate(9, 11), result);
    }

    @Test
    void scatter_northwest_decreasesBoth() {
        FieldCoordinate result = ScatterCalc.scatterCoordinate(new FieldCoordinate(10, 10), Direction.NORTHWEST, 1);
        assertEquals(new FieldCoordinate(9, 9), result);
    }

    @Test
    void scatter_distanceTwo_doublesOffset() {
        FieldCoordinate result = ScatterCalc.scatterCoordinate(new FieldCoordinate(5, 5), Direction.NORTHEAST, 2);
        assertEquals(new FieldCoordinate(7, 3), result);
    }

    @Test
    void scatter_distanceZero_returnsStart() {
        FieldCoordinate start = new FieldCoordinate(7, 7);
        FieldCoordinate result = ScatterCalc.scatterCoordinate(start, Direction.SOUTH, 0);
        assertEquals(start, result);
    }
}
