package com.fumbbl.ffb.server.util;

/**
 * Pure post-match and kickoff-event roll calculations extracted from DiceInterpreter.
 */
public final class PostMatchCalc {

    /**
     * Interpret the fan factor roll result at the end of a game.
     *
     * @param rollTotal    Sum of the fan factor dice (usually 3D6).
     * @param fanFactor    The team's fan factor rating.
     * @param scoreDiff    (team score - opponent score): positive = winning, negative = losing, 0 = draw.
     * @return +1 if winning/drawing AND rollTotal > fanFactor; -1 if losing/drawing AND rollTotal < fanFactor; 0 otherwise.
     */
    public static int interpretFanFactorRoll(int rollTotal, int fanFactor, int scoreDiff) {
        if (scoreDiff >= 0 && rollTotal > fanFactor) return 1;
        if (scoreDiff <= 0 && rollTotal < fanFactor) return -1;
        return 0;
    }

    /**
     * Interpret a Master Chef roll: each die that shows 4, 5, or 6 steals a reroll from the opponent.
     *
     * @param dice individual die results (usually 3 dice)
     * @return number of rerolls stolen (0 to dice.length)
     */
    public static int interpretMasterChefRoll(int... dice) {
        int stolen = 0;
        for (int d : dice) {
            if (d > 3) stolen++;
        }
        return stolen;
    }

    private PostMatchCalc() {}
}
