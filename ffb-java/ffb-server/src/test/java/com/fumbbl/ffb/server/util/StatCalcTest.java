package com.fumbbl.ffb.server.util;

import com.fumbbl.ffb.RulesCollection.Rules;
import com.fumbbl.ffb.modifiers.PlayerStatKey;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.CsvSource;

import static org.junit.jupiter.api.Assertions.*;

class StatCalcTest {

    // ── statMin ───────────────────────────────────────────────────────────────

    @Test
    void statMin_BB2016_all_stats_return_1() {
        for (PlayerStatKey key : PlayerStatKey.values()) {
            assertEquals(1, StatCalc.statMin(key, Rules.BB2016), "Expected min=1 for " + key);
        }
    }

    @ParameterizedTest
    @CsvSource({"BB2020", "BB2025"})
    void statMin_modern_AV_returns_3(Rules rules) {
        assertEquals(3, StatCalc.statMin(PlayerStatKey.AV, rules));
    }

    @ParameterizedTest
    @CsvSource({"BB2020", "BB2025"})
    void statMin_modern_non_AV_returns_1(Rules rules) {
        assertEquals(1, StatCalc.statMin(PlayerStatKey.MA, rules));
        assertEquals(1, StatCalc.statMin(PlayerStatKey.ST, rules));
        assertEquals(1, StatCalc.statMin(PlayerStatKey.AG, rules));
        assertEquals(1, StatCalc.statMin(PlayerStatKey.PA, rules));
    }

    // ── statMax ───────────────────────────────────────────────────────────────

    @Test
    void statMax_BB2016_MA_ST_AG_AV_return_10() {
        assertEquals(10, StatCalc.statMax(PlayerStatKey.MA, Rules.BB2016));
        assertEquals(10, StatCalc.statMax(PlayerStatKey.ST, Rules.BB2016));
        assertEquals(10, StatCalc.statMax(PlayerStatKey.AG, Rules.BB2016));
        assertEquals(10, StatCalc.statMax(PlayerStatKey.AV, Rules.BB2016));
    }

    @ParameterizedTest
    @CsvSource({"BB2020", "BB2025"})
    void statMax_modern_MA_returns_9(Rules rules) {
        assertEquals(9, StatCalc.statMax(PlayerStatKey.MA, rules));
    }

    @ParameterizedTest
    @CsvSource({"BB2020", "BB2025"})
    void statMax_modern_ST_returns_8(Rules rules) {
        assertEquals(8, StatCalc.statMax(PlayerStatKey.ST, rules));
    }

    @ParameterizedTest
    @CsvSource({"BB2020", "BB2025"})
    void statMax_modern_AG_returns_6(Rules rules) {
        assertEquals(6, StatCalc.statMax(PlayerStatKey.AG, rules));
    }

    @ParameterizedTest
    @CsvSource({"BB2020", "BB2025"})
    void statMax_modern_PA_returns_6(Rules rules) {
        assertEquals(6, StatCalc.statMax(PlayerStatKey.PA, rules));
    }

    @ParameterizedTest
    @CsvSource({"BB2020", "BB2025"})
    void statMax_modern_AV_returns_11(Rules rules) {
        assertEquals(11, StatCalc.statMax(PlayerStatKey.AV, rules));
    }

    // ── applyLastingInjury ────────────────────────────────────────────────────

    @Test
    void applyLastingInjury_BB2016_MA_decreases() {
        assertEquals(5, StatCalc.applyLastingInjury(6, PlayerStatKey.MA, Rules.BB2016));
    }

    @Test
    void applyLastingInjury_BB2016_AG_decreases() {
        assertEquals(3, StatCalc.applyLastingInjury(4, PlayerStatKey.AG, Rules.BB2016));
    }

    @Test
    void applyLastingInjury_BB2016_MA_floored_at_1() {
        assertEquals(1, StatCalc.applyLastingInjury(1, PlayerStatKey.MA, Rules.BB2016));
    }

    @ParameterizedTest
    @CsvSource({"BB2020", "BB2025"})
    void applyLastingInjury_modern_AG_increases(Rules rules) {
        assertEquals(4, StatCalc.applyLastingInjury(3, PlayerStatKey.AG, rules));
    }

    @ParameterizedTest
    @CsvSource({"BB2020", "BB2025"})
    void applyLastingInjury_modern_PA_increases(Rules rules) {
        assertEquals(5, StatCalc.applyLastingInjury(4, PlayerStatKey.PA, rules));
    }

    @ParameterizedTest
    @CsvSource({"BB2020", "BB2025"})
    void applyLastingInjury_modern_AG_capped_at_max(Rules rules) {
        assertEquals(6, StatCalc.applyLastingInjury(6, PlayerStatKey.AG, rules));
    }

    @ParameterizedTest
    @CsvSource({"BB2020", "BB2025"})
    void applyLastingInjury_modern_MA_decreases(Rules rules) {
        assertEquals(5, StatCalc.applyLastingInjury(6, PlayerStatKey.MA, rules));
    }

    @ParameterizedTest
    @CsvSource({"BB2020", "BB2025"})
    void applyLastingInjury_modern_AV_floored_at_3(Rules rules) {
        assertEquals(3, StatCalc.applyLastingInjury(3, PlayerStatKey.AV, rules));
    }

    // ── applyInGameAgilityInjury ──────────────────────────────────────────────

    @Test
    void applyInGameAgilityInjury_BB2016_decreases() {
        assertEquals(3, StatCalc.applyInGameAgilityInjury(4, 1, Rules.BB2016));
        assertEquals(2, StatCalc.applyInGameAgilityInjury(4, 2, Rules.BB2016));
    }

    @ParameterizedTest
    @CsvSource({"BB2020", "BB2025"})
    void applyInGameAgilityInjury_modern_increases(Rules rules) {
        assertEquals(5, StatCalc.applyInGameAgilityInjury(4, 1, rules));
        assertEquals(6, StatCalc.applyInGameAgilityInjury(4, 2, rules));
    }

    // ── statCanBeReducedByInjury ──────────────────────────────────────────────

    @Test
    void statCanBeReduced_BB2016_no_prior_injuries_true() {
        assertTrue(StatCalc.statCanBeReducedByInjury(4, 4, Rules.BB2016));
    }

    @Test
    void statCanBeReduced_BB2016_one_prior_injury_true() {
        assertTrue(StatCalc.statCanBeReducedByInjury(4, 3, Rules.BB2016));
    }

    @Test
    void statCanBeReduced_BB2016_two_prior_injuries_false() {
        assertFalse(StatCalc.statCanBeReducedByInjury(4, 2, Rules.BB2016));
    }

    @ParameterizedTest
    @CsvSource({"BB2020", "BB2025"})
    void statCanBeReduced_modern_always_true(Rules rules) {
        assertTrue(StatCalc.statCanBeReducedByInjury(4, 4, rules));
        assertTrue(StatCalc.statCanBeReducedByInjury(4, 3, rules));
        assertTrue(StatCalc.statCanBeReducedByInjury(4, 2, rules));
        assertTrue(StatCalc.statCanBeReducedByInjury(4, 1, rules));
    }
}
