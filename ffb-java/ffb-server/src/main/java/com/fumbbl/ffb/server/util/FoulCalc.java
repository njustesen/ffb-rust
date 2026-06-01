package com.fumbbl.ffb.server.util;

/**
 * Pure foul-mechanics calculations.
 * All functions are stateless and require no game context.
 */
public final class FoulCalc {

    private FoulCalc() {}

    /**
     * Determine whether the referee spots a foul based on the armor roll.
     * Referee spots the foul if the two armor dice show the same value (doubles),
     * unless the fouler has SneakyGit (which suppresses the armor-roll detection).
     *
     * @param armorDie1    first armor die result
     * @param armorDie2    second armor die result
     * @param hasSneakyGit true if the fouling player has the SneakyGit skill
     * @return true if the foul is spotted by the armor roll
     */
    public static boolean isSpottedByArmorRoll(int armorDie1, int armorDie2, boolean hasSneakyGit) {
        return (armorDie1 == armorDie2) && !hasSneakyGit;
    }

    /**
     * Determine whether the referee spots a foul based on the injury roll.
     * When the armor was broken, doubles on the injury roll are also spotted
     * regardless of whether the fouler has SneakyGit.
     *
     * @param injuryDie1  first injury die result
     * @param injuryDie2  second injury die result
     * @param armorBroken true if the armor was broken (injury roll was made)
     * @return true if the foul is spotted by the injury roll
     */
    public static boolean isSpottedByInjuryRoll(int injuryDie1, int injuryDie2, boolean armorBroken) {
        return armorBroken && (injuryDie1 == injuryDie2);
    }

    /**
     * Determine whether the referee spots the foul overall.
     * Spotted if either the armor roll or the injury roll triggered detection.
     *
     * @param armorDie1    first armor die
     * @param armorDie2    second armor die
     * @param injuryDie1   first injury die (ignored if armorBroken is false)
     * @param injuryDie2   second injury die (ignored if armorBroken is false)
     * @param armorBroken  whether the armor was broken
     * @param hasSneakyGit whether the fouler has SneakyGit
     * @return true if the referee spots the foul
     */
    public static boolean isSpottedByReferee(
            int armorDie1, int armorDie2,
            int injuryDie1, int injuryDie2,
            boolean armorBroken,
            boolean hasSneakyGit) {
        return isSpottedByArmorRoll(armorDie1, armorDie2, hasSneakyGit)
            || isSpottedByInjuryRoll(injuryDie1, injuryDie2, armorBroken);
    }

    /**
     * Minimum armor value to break armor in a foul (same formula as normal armor break:
     * armor roll total must strictly exceed the player's AV).
     *
     * @param armourValue player's AV stat
     * @return minimum 2D6 total needed to break armor
     */
    public static int minimumRollToBreakArmour(int armourValue) {
        return armourValue + 1;
    }
}
