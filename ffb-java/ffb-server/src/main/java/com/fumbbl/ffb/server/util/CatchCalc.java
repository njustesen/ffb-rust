package com.fumbbl.ffb.server.util;

import com.fumbbl.ffb.RulesCollection.Rules;

/**
 * Pure catch and interception roll target calculations, edition-aware.
 *
 * BB2016: target = max(2, (7 - min(AG, 6)) + modifiers)
 *   No action-type adjustment for catch (uses base, unlike dodge which subtracts 1).
 *   Interception adds +2 to the base.
 *
 * BB2020/BB2025: target = max(2, AG + modifiers)
 *   All action types (catch, interception) use the same direct formula.
 *
 * Modifier sign convention: positive = harder (penalty), negative = easier (benefit).
 */
public final class CatchCalc {

    private CatchCalc() {}

    /**
     * Minimum roll for a catch (BB2016).
     *
     * @param agility       player's agility stat
     * @param modifierTotal sum of all modifier values
     */
    public static int minimumRollCatchBB2016(int agility, int modifierTotal) {
        return Math.max(2, AgilityCalc.agilityRollBaseBB2016(agility) + modifierTotal);
    }

    /**
     * Minimum roll for an interception (BB2016).
     * Interception is harder by +2.
     *
     * @param agility       interceptor's agility
     * @param modifierTotal sum of all modifier values
     */
    public static int minimumRollInterceptionBB2016(int agility, int modifierTotal) {
        return Math.max(2, AgilityCalc.agilityRollBaseBB2016(agility) + 2 + modifierTotal);
    }

    /**
     * Minimum roll for a catch in BB2020/BB2025.
     *
     * @param agility       player's agility (the "X+" target value)
     * @param modifierTotal sum of all modifier values
     */
    public static int minimumRollCatchBB2020(int agility, int modifierTotal) {
        return Math.max(2, agility + modifierTotal);
    }

    /**
     * Minimum roll for an interception in BB2020/BB2025.
     * Same formula as catch (unlike BB2016 where interception has +2 penalty).
     *
     * @param agility       interceptor's agility
     * @param modifierTotal sum of all modifier values
     */
    public static int minimumRollInterceptionBB2020(int agility, int modifierTotal) {
        return Math.max(2, agility + modifierTotal);
    }

    /**
     * Minimum roll for a catch, edition-dispatched.
     */
    public static int minimumRollCatch(int agility, int modifierTotal, Rules rules) {
        if (rules == Rules.BB2016) {
            return minimumRollCatchBB2016(agility, modifierTotal);
        } else {
            return minimumRollCatchBB2020(agility, modifierTotal);
        }
    }

    /**
     * Minimum roll for an interception, edition-dispatched.
     */
    public static int minimumRollInterception(int agility, int modifierTotal, Rules rules) {
        if (rules == Rules.BB2016) {
            return minimumRollInterceptionBB2016(agility, modifierTotal);
        } else {
            return minimumRollInterceptionBB2020(agility, modifierTotal);
        }
    }
}
