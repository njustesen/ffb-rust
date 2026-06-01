package com.fumbbl.ffb.server.util;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class PostMatchCalcTest {

    // ── interpretFanFactorRoll ────────────────────────────────────────────────

    @Test
    void winning_rollHigherThanFF_returns1() {
        // scoreDiff > 0 (winning), roll beats fan factor
        assertEquals(1, PostMatchCalc.interpretFanFactorRoll(10, 8, 1));
    }

    @Test
    void winning_rollEqualToFF_returns0() {
        assertEquals(0, PostMatchCalc.interpretFanFactorRoll(8, 8, 1));
    }

    @Test
    void winning_rollLowerThanFF_returns0() {
        // Not losing, so cannot get -1
        assertEquals(0, PostMatchCalc.interpretFanFactorRoll(6, 8, 1));
    }

    @Test
    void losing_rollLowerThanFF_returnsMinus1() {
        assertEquals(-1, PostMatchCalc.interpretFanFactorRoll(6, 8, -1));
    }

    @Test
    void losing_rollHigherThanFF_returns0() {
        assertEquals(0, PostMatchCalc.interpretFanFactorRoll(10, 8, -1));
    }

    @Test
    void draw_rollHigherThanFF_returns1() {
        // scoreDiff == 0 satisfies both >= 0 conditions
        assertEquals(1, PostMatchCalc.interpretFanFactorRoll(10, 8, 0));
    }

    @Test
    void draw_rollLowerThanFF_returnsMinus1() {
        assertEquals(-1, PostMatchCalc.interpretFanFactorRoll(6, 8, 0));
    }

    @Test
    void draw_rollEqualToFF_returns0() {
        assertEquals(0, PostMatchCalc.interpretFanFactorRoll(8, 8, 0));
    }

    // ── interpretMasterChefRoll ───────────────────────────────────────────────

    @Test
    void allLow_stealsNothing() {
        assertEquals(0, PostMatchCalc.interpretMasterChefRoll(1, 2, 3));
    }

    @Test
    void allHigh_stealsAll() {
        assertEquals(3, PostMatchCalc.interpretMasterChefRoll(4, 5, 6));
    }

    @Test
    void mixed_stealsPartial() {
        assertEquals(2, PostMatchCalc.interpretMasterChefRoll(3, 4, 6));
    }

    @Test
    void singleDieHigh_steals1() {
        assertEquals(1, PostMatchCalc.interpretMasterChefRoll(4));
    }

    @Test
    void singleDieLow_steals0() {
        assertEquals(0, PostMatchCalc.interpretMasterChefRoll(3));
    }

    @Test
    void emptyDice_steals0() {
        assertEquals(0, PostMatchCalc.interpretMasterChefRoll());
    }
}
