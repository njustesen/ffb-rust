package com.fumbbl.ffb.server.util;

import com.fumbbl.ffb.RulesCollection.Rules;

/**
 * Pure roll-result calculations extracted from DiceInterpreter and StatsMechanic.
 *
 * All methods are stateless and take primitive inputs so they can be tested
 * without any Game or GameState context.
 */
public final class RollCalc {

    private RollCalc() {}

    /**
     * Whether a single d6 skill roll (dodge, pickup, catch, etc.) succeeds.
     *
     * Blood Bowl rule: a natural 6 always succeeds regardless of minimum roll;
     * a natural 1 always fails; otherwise the roll must meet or beat the minimum.
     */
    public static boolean isSkillRollSuccessful(int roll, int minimumRoll) {
        return (roll == 6) || ((roll != 1) && (roll >= minimumRoll));
    }

    /**
     * Whether a 2d6 armour roll breaks the player's armour.
     *
     * BB2016: rollTotal must strictly exceed armour (roll > armour).
     * BB2020/BB2025: rollTotal must equal or exceed armour (roll >= armour).
     *
     * @param armour     the player's effective armour value (after any fixed-reduction effects)
     * @param rollTotal  sum of both armour dice plus all modifiers
     * @param rules      edition determines the comparison operator
     */
    public static boolean isArmourBroken(int armour, int rollTotal, Rules rules) {
        if (rules == Rules.BB2016) {
            return rollTotal > armour;
        } else {
            return rollTotal >= armour;
        }
    }

    /**
     * Applies the Chainsaw (or similar) fixed-armour-reduction effect.
     *
     * Certain skills reduce the target's armour to a fixed cap before the comparison.
     * BB2016 cap is 7; BB2020/BB2025 cap is 8.
     *
     * @param armour  the player's base armour value
     * @param rules   edition determines the cap
     */
    public static int applyFixedArmourReduction(int armour, Rules rules) {
        int cap = (rules == Rules.BB2016) ? 7 : 8;
        return Math.min(armour, cap);
    }

    /**
     * Minimum roll required for a Going For It attempt.
     *
     * Base is always 2; positive modifiers increase it but it is capped at a minimum
     * of 2 (negative modifiers cannot push below 2).
     *
     * @param modifierTotal  sum of all GFI modifiers (positive = harder, negative = easier)
     */
    public static int minimumRollGoingForIt(int modifierTotal) {
        return Math.max(2, 2 + modifierTotal);
    }
}
