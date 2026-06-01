package com.fumbbl.ffb.server.util;

/**
 * Pure block dice count calculation extracted from ServerUtilBlock, testable without GameState.
 *
 * Input strengths are pre-computed totals (base + assists after all skill/edition modifiers).
 * Positive result → attacker picks dice. Negative → defender picks dice.
 */
public final class BlockDiceCalc {

    private BlockDiceCalc() {}

    /**
     * Returns the number of block dice given final attacker and defender strength totals.
     *
     * Mirrors ServerUtilBlock.findNrOfBlockDice() comparison logic exactly:
     *   attacker > 2× defender  →  +3
     *   attacker >    defender  →  +2
     *   equal                   →  +1
     *   attacker <    defender  →  -2
     *   attacker < 0.5× defender → -3
     */
    public static int blockDiceCount(int attackerStr, int defenderStr) {
        if (attackerStr > 2 * defenderStr) return 3;
        if (attackerStr > defenderStr) return 2;
        if (2 * attackerStr < defenderStr) return -3;
        if (attackerStr < defenderStr) return -2;
        return 1;
    }

    /**
     * Applies the "add block die" skill bonus (e.g. Horns during blitz).
     * Only triggers when the current count is 1 or 2 (cannot exceed 3, no effect on negative).
     */
    public static int applyAddBlockDie(int dice) {
        if (dice == 1 || dice == 2) return dice + 1;
        return dice;
    }

    /** BB2016 multi-block: defender strength +2, attacker unchanged. */
    public static int multiBlockDefenderModifierBB2016() { return 2; }

    /** BB2016 multi-block: attacker strength unchanged. */
    public static int multiBlockAttackerModifierBB2016() { return 0; }

    /** BB2020/BB2025 multi-block: attacker strength -2. */
    public static int multiBlockAttackerModifierBB2020() { return -2; }

    /** BB2020/BB2025 multi-block: defender strength unchanged. */
    public static int multiBlockDefenderModifierBB2020() { return 0; }
}
