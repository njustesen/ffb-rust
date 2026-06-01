package com.fumbbl.ffb.server.util;

import com.fumbbl.ffb.RulesCollection.Rules;

/**
 * Pure agility-roll target calculations, edition-aware.
 *
 * BB2016 uses a conversion table: base = 7 - min(AG, 6).
 *   Dodge/Pickup: base - 1 + modifiers (floor 2)
 *   Catch/Jump-up/Leap/Gaze: base + modifiers (floor 2)
 *   Interception: base + 2 + modifiers (floor 2)
 *
 * BB2020/BB2025 use the agility stat directly as the target:
 *   All rolls: ag + modifiers (floor 2)
 *   (where AG is the "X+" value, e.g., "3+" means ag=3)
 */
public final class AgilityCalc {

    private AgilityCalc() {}

    /**
     * BB2016 base target for agility rolls before action-specific adjustment.
     *
     * Agility 1 → 6, 2 → 5, 3 → 4, 4 → 3, 5 → 2, 6+ → 1.
     */
    public static int agilityRollBaseBB2016(int agility) {
        return 7 - Math.min(agility, 6);
    }

    /**
     * Minimum roll for a dodge or pickup (BB2016).
     * Dodge and pickup share the same formula (base - 1, not base).
     *
     * @param agility       player's current agility value (raw stat)
     * @param modifierTotal sum of all applicable modifiers
     */
    public static int minimumRollDodgeBB2016(int agility, int modifierTotal) {
        return Math.max(2, agilityRollBaseBB2016(agility) - 1 + modifierTotal);
    }

    /**
     * Minimum roll for a catch (BB2016).
     * Catch uses the base without the -1 dodge bonus.
     */
    public static int minimumRollCatchBB2016(int agility, int modifierTotal) {
        return Math.max(2, agilityRollBaseBB2016(agility) + modifierTotal);
    }

    /**
     * Minimum roll for a jump-up, leap, or hypnotic gaze (BB2016).
     * Uses the base without adjustment.
     */
    public static int minimumRollBaseBB2016(int agility, int modifierTotal) {
        return Math.max(2, agilityRollBaseBB2016(agility) + modifierTotal);
    }

    /**
     * Minimum roll for an interception (BB2016).
     * Interception is harder by +2.
     */
    public static int minimumRollInterceptionBB2016(int agility, int modifierTotal) {
        return Math.max(2, agilityRollBaseBB2016(agility) + 2 + modifierTotal);
    }

    /**
     * Minimum roll for any agility-based action (BB2020/BB2025).
     *
     * In BB2020, the agility stat is the target number directly ("3+" → ag=3).
     * All action types (dodge, catch, pickup, intercept) use the same formula.
     *
     * @param agility       player's agility (already the target number)
     * @param modifierTotal sum of all applicable modifiers
     */
    public static int minimumRollBB2020(int agility, int modifierTotal) {
        return Math.max(2, agility + modifierTotal);
    }

    /**
     * Minimum roll for any agility-based action, edition-dispatched.
     *
     * For BB2016, uses the dodge/pickup formula (base - 1 + mods).
     * For BB2020/BB2025, uses the direct formula (ag + mods).
     *
     * Use the specific methods above for catch/interception in BB2016 since
     * those use different bases.
     *
     * @param agility       player's agility stat
     * @param modifierTotal sum of all applicable modifiers
     * @param rules         which edition
     */
    public static int minimumRollDodge(int agility, int modifierTotal, Rules rules) {
        if (rules == Rules.BB2016) {
            return minimumRollDodgeBB2016(agility, modifierTotal);
        } else {
            return minimumRollBB2020(agility, modifierTotal);
        }
    }
}
