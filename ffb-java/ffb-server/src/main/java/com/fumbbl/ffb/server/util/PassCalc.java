package com.fumbbl.ffb.server.util;

import com.fumbbl.ffb.PassingDistance;
import com.fumbbl.ffb.RulesCollection.Rules;

/**
 * Pure pass-roll target calculations, edition-aware.
 *
 * BB2016 (agility-based):
 *   target = max(max(2 - (dist_mod - mods), 2), 7 - min(ag, 6) - dist_mod + mods)
 *   The fumble boundary constraint: max(2 - dist_mod + mods, ...) ensures the target
 *   is always above the fumble line even for high-AG players throwing long bombs.
 *
 * BB2020/BB2025 (PA-based):
 *   target = max(2, pa + dist_mod + mods)
 *   Returns null if the player has no passing ability (pa == 0).
 */
public final class PassCalc {

    private PassCalc() {}

    /**
     * Minimum roll for a pass in BB2016.
     *
     * @param agility       player's agility (raw stat)
     * @param distance      pass distance category
     * @param modifierTotal sum of all modifier values (positive = harder, negative = easier)
     */
    public static int minimumRollPassBB2016(int agility, PassingDistance distance, int modifierTotal) {
        int distMod = distance.getModifier2016();
        int agCapped = Math.min(agility, 6);
        int agBased = 7 - agCapped - distMod + modifierTotal;
        int fumbleBoundary = 2 - distMod + modifierTotal;
        return Math.max(Math.max(agBased, fumbleBoundary), 2);
    }

    /**
     * Minimum roll for a pass in BB2020/BB2025.
     *
     * Returns null when the player cannot pass (pa == 0).
     *
     * @param passingAbility player's PA value (already the target number, e.g. 3 for "3+")
     * @param distance       pass distance category
     * @param modifierTotal  sum of all modifier values
     */
    public static Integer minimumRollPassBB2020(int passingAbility, PassingDistance distance, int modifierTotal) {
        if (passingAbility <= 0) {
            return null;
        }
        return Math.max(2, passingAbility + distance.getModifier2020() + modifierTotal);
    }

    /**
     * Whether a BB2016 pass roll is a modified fumble.
     *
     * Fumble when: roll + dist_mod - modifiers <= 1
     * (i.e. the modified result is 1 or less).
     */
    public static boolean isModifiedFumbleBB2016(int roll, PassingDistance distance, int modifierTotal) {
        return (roll + distance.getModifier2016() - modifierTotal) <= 1;
    }
}
