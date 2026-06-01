package com.fumbbl.ffb.server.util;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.CsvSource;

import static org.junit.jupiter.api.Assertions.assertEquals;

class BlockDiceCalcTest {

    // ── Equal strength ────────────────────────────────────────────────────────

    @Test
    void equalStrength_returnsOneDie() {
        assertEquals(1, BlockDiceCalc.blockDiceCount(3, 3));
    }

    @Test
    void equalStrength_atEdge1vs1_returnsOneDie() {
        assertEquals(1, BlockDiceCalc.blockDiceCount(1, 1));
    }

    // ── Attacker advantage ───────────────────────────────────────────────────

    @ParameterizedTest(name = "attacker={0} defender={1} → 2 dice")
    @CsvSource({"4,3", "5,4", "5,3", "3,2", "6,4"})
    void attackerStronger_returnsTwoDice(int attacker, int defender) {
        assertEquals(2, BlockDiceCalc.blockDiceCount(attacker, defender));
    }

    @ParameterizedTest(name = "attacker={0} defender={1} → 3 dice")
    @CsvSource({"7,3", "6,2", "4,1", "10,4", "8,3"})
    void attackerDoubleStrength_returnsThreeDice(int attacker, int defender) {
        assertEquals(3, BlockDiceCalc.blockDiceCount(attacker, defender));
    }

    @Test
    void attackerExactlyDoubleDefender_returnsThreeDice() {
        // 6 > 2×3 → false (strictly greater), so 3 dice requires strictly more than double
        assertEquals(2, BlockDiceCalc.blockDiceCount(6, 3));
    }

    @Test
    void attackerOneAboveDouble_returnsThreeDice() {
        assertEquals(3, BlockDiceCalc.blockDiceCount(7, 3));
    }

    // ── Defender advantage ───────────────────────────────────────────────────

    @ParameterizedTest(name = "attacker={0} defender={1} → -2 dice")
    @CsvSource({"3,4", "2,3", "4,5", "3,5", "3,6"})
    void defenderStronger_returnsMinusTwoDice(int attacker, int defender) {
        assertEquals(-2, BlockDiceCalc.blockDiceCount(attacker, defender));
    }

    @Test
    void defenderDoubleAttacker_returnsMinusThreeDice() {
        assertEquals(-3, BlockDiceCalc.blockDiceCount(3, 7));
    }

    @Test
    void defenderExactlyDoubleAttacker_returnsMinusTwoDice() {
        // 2×3 < 6 → false (strictly less), so -3 requires strictly more than double
        assertEquals(-2, BlockDiceCalc.blockDiceCount(3, 6));
    }

    @ParameterizedTest(name = "attacker={0} defender={1} → -3 dice")
    @CsvSource({"1,3", "2,5", "3,7", "4,9"})
    void defenderMoreThanDoubleAttacker_returnsMinusThreeDice(int attacker, int defender) {
        assertEquals(-3, BlockDiceCalc.blockDiceCount(attacker, defender));
    }

    // ── addBlockDie bonus ────────────────────────────────────────────────────

    @Test
    void addBlockDie_onOneDie_returnsTwo() {
        assertEquals(2, BlockDiceCalc.applyAddBlockDie(1));
    }

    @Test
    void addBlockDie_onTwoDice_returnsThree() {
        assertEquals(3, BlockDiceCalc.applyAddBlockDie(2));
    }

    @Test
    void addBlockDie_onThreeDice_noChange() {
        assertEquals(3, BlockDiceCalc.applyAddBlockDie(3));
    }

    @Test
    void addBlockDie_onNegativeDice_noChange() {
        assertEquals(-2, BlockDiceCalc.applyAddBlockDie(-2));
        assertEquals(-3, BlockDiceCalc.applyAddBlockDie(-3));
    }

    // ── Multi-block modifiers ─────────────────────────────────────────────────

    @Test
    void bb2016_multiBlock_defenderGetsPlus2() {
        assertEquals(2, BlockDiceCalc.multiBlockDefenderModifierBB2016());
        assertEquals(0, BlockDiceCalc.multiBlockAttackerModifierBB2016());
    }

    @Test
    void bb2020_multiBlock_attackerGetsMinus2() {
        assertEquals(-2, BlockDiceCalc.multiBlockAttackerModifierBB2020());
        assertEquals(0, BlockDiceCalc.multiBlockDefenderModifierBB2020());
    }

    @Test
    void multiBlock_bb2016_reducesAttackerAdvantage() {
        // ST4 attacker vs ST3 defender → normally 2 dice
        // BB2016 multi-block: defender +2 → ST3+2=5, attacker ST4 < ST5 → -2 dice
        int defStr = 3 + BlockDiceCalc.multiBlockDefenderModifierBB2016();
        int attStr = 4 + BlockDiceCalc.multiBlockAttackerModifierBB2016();
        assertEquals(-2, BlockDiceCalc.blockDiceCount(attStr, defStr));
    }

    @Test
    void multiBlock_bb2020_reducesAttackerStrength() {
        // ST4 attacker vs ST3 defender → normally 2 dice
        // BB2020 multi-block: attacker -2 → ST4-2=2 vs ST3 → -2 dice
        int defStr = 3 + BlockDiceCalc.multiBlockDefenderModifierBB2020();
        int attStr = 4 + BlockDiceCalc.multiBlockAttackerModifierBB2020();
        assertEquals(-2, BlockDiceCalc.blockDiceCount(attStr, defStr));
    }
}
