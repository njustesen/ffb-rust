package com.fumbbl.ffb.server.mechanic;

import com.fumbbl.ffb.RulesCollection.Rules;

/**
 * Pure SPP (Star Player Points) award values by edition.
 *
 * BB2016:  TD=3, CAS=2, COMP=1, INT=2, DEFL=1, CATCH=1, LANDING=0, MVP=5
 * BB2020:  TD=3, CAS=2, COMP=1, INT=2, DEFL=1, CATCH=1, LANDING=0, MVP=4
 *          Teams can earn +1 additional for CAS/COMP/CATCH (league setting)
 * BB2025:  Same as BB2020 but LANDING=1; BrawlinBrutes: TD=2, CAS=3
 *
 * The Brawlin' Brutes special rule is handled externally (caller passes isBrawlinBrutes).
 */
public final class SppCalc {

    private SppCalc() {}

    public static int touchdownSpp(Rules rules, boolean isBrawlinBrutes) {
        if (rules == Rules.BB2025 && isBrawlinBrutes) return 2;
        return 3;
    }

    public static int touchdownSpp(Rules rules) {
        return touchdownSpp(rules, false);
    }

    public static int casualtySpp(Rules rules, boolean isBrawlinBrutes) {
        if (rules == Rules.BB2025 && isBrawlinBrutes) return 3;
        return 2;
    }

    public static int casualtySpp(Rules rules) {
        return casualtySpp(rules, false);
    }

    public static int completionSpp() {
        return 1; // same across all editions
    }

    public static int interceptionSpp() {
        return 2; // same across all editions
    }

    public static int deflectionSpp() {
        return 1; // same across all editions
    }

    public static int catchSpp() {
        return 1; // same across all editions
    }

    public static int landingSpp(Rules rules) {
        return (rules == Rules.BB2025) ? 1 : 0;
    }

    public static int mvpSpp(Rules rules) {
        return (rules == Rules.BB2016) ? 5 : 4;
    }

    /**
     * Additional SPP awarded per casualty/completion/catch when the team has the
     * league "additional SPP" bonus (BB2020/BB2025 only).
     */
    public static int additionalSpp(Rules rules) {
        return (rules == Rules.BB2016) ? 0 : 1;
    }

    /**
     * SPP thresholds at which a player advances to the next level.
     * Applies to BB2016 only (BB2020/BB2025 level by skills gained).
     *
     * 6, 16, 31, 51, 76, 176 (inclusive, i.e. ≥ 6 = Experienced)
     */
    public static final int[] LEVEL_THRESHOLDS_BB2016 = { 6, 16, 31, 51, 76, 176 };

    /**
     * Player level (0=Rookie, 1=Experienced, ..., 6=Legend) from SPP total (BB2016 only).
     */
    public static int playerLevelBB2016(int currentSpp) {
        int level = 0;
        for (int threshold : LEVEL_THRESHOLDS_BB2016) {
            if (currentSpp >= threshold) level++;
            else break;
        }
        return level;
    }

    /**
     * Whether the player just levelled up given old and new SPP totals (BB2016).
     */
    public static boolean justLevelledUpBB2016(int oldSpp, int newSpp) {
        return playerLevelBB2016(oldSpp) < playerLevelBB2016(newSpp);
    }
}
