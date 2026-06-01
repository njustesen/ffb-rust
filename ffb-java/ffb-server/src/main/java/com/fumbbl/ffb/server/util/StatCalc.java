package com.fumbbl.ffb.server.util;

import com.fumbbl.ffb.RulesCollection.Rules;
import com.fumbbl.ffb.modifiers.PlayerStatKey;

/**
 * Pure player stat limit and lasting-injury calculations.
 * Mirrors Java StatsMechanic implementations for each edition.
 */
public final class StatCalc {

    // ── Stat limits ───────────────────────────────────────────────────────────

    /**
     * Minimum allowed value for the given stat in the given edition.
     * BB2016: all stats minimum 1.
     * BB2020/BB2025: AV minimum 3; all others minimum 1.
     */
    public static int statMin(PlayerStatKey key, Rules rules) {
        if (rules == Rules.BB2020 || rules == Rules.BB2025) {
            return (key == PlayerStatKey.AV) ? 3 : 1;
        }
        return 1;
    }

    /**
     * Maximum allowed value for the given stat in the given edition.
     * BB2016: all stats maximum 10.
     * BB2020/BB2025: MA=9, ST=8, AG=6, PA=6, AV=11.
     */
    public static int statMax(PlayerStatKey key, Rules rules) {
        if (rules == Rules.BB2020 || rules == Rules.BB2025) {
            switch (key) {
            case MA: return 9;
            case ST: return 8;
            case AG: return 6;
            case PA: return 6;
            case AV: return 11;
            default: return 0;
            }
        }
        // BB2016: all stats cap at 10
        switch (key) {
        case MA: case ST: case AG: case AV: return 10;
        default: return 0;
        }
    }

    /**
     * Apply a lasting injury (post-game) to a stat value, clamped to edition limits.
     *
     * BB2016: all stats -1 (floored at minimum).
     * BB2020/BB2025: AG and PA +1 (worse target number, floored at max); all others -1.
     */
    public static int applyLastingInjury(int value, PlayerStatKey key, Rules rules) {
        int min = statMin(key, rules);
        int max = statMax(key, rules);
        if (rules == Rules.BB2020 || rules == Rules.BB2025) {
            if (key == PlayerStatKey.AG || key == PlayerStatKey.PA) {
                return Math.min(value + 1, max);
            }
            return Math.max(value - 1, min);
        }
        // BB2016: all decrease
        return Math.max(value - 1, min);
    }

    /**
     * Apply an in-game agility injury (e.g. from Niggling Injury) to the current agility value.
     * BB2016: agility decreases (higher = better in BB2016).
     * BB2020/BB2025: agility target number increases (higher = worse).
     */
    public static int applyInGameAgilityInjury(int agility, int decreases, Rules rules) {
        if (rules == Rules.BB2020 || rules == Rules.BB2025) {
            return agility + decreases;
        }
        return agility - decreases;
    }

    /**
     * Whether a stat value can be further reduced by an in-game injury.
     * BB2016: only if fewer than 2 injuries already applied (prevents going below minimum too fast).
     * BB2020/BB2025: always reducible.
     */
    public static boolean statCanBeReducedByInjury(int originalValue, int currentValue, Rules rules) {
        if (rules == Rules.BB2016) {
            return (originalValue - currentValue) < 2;
        }
        return true;
    }

    private StatCalc() {}
}
