package com.fumbbl.ffb.server.util;

/**
 * Pure kickoff-event roll calculations (Cheering Fans, Brilliant Coaching).
 *
 * Both kickoff events compare two team totals. In case of a tie, BOTH teams win a reroll.
 */
public final class KickoffEventCalc {

    /**
     * Cheering Fans total for one team: D6 roll + fame + cheerleaders.
     */
    public static int cheeringFansTotal(int dieRoll, int fame, int cheerleaders) {
        return dieRoll + fame + cheerleaders;
    }

    /**
     * Brilliant Coaching total for one team: D6 roll + fame + assistantCoaches - (coachBanned ? 1 : 0).
     */
    public static int brilliantCoachingTotal(int dieRoll, int fame, int assistantCoaches, boolean coachBanned) {
        return dieRoll + fame + assistantCoaches + (coachBanned ? -1 : 0);
    }

    /**
     * Whether a team gains a reroll: true when its total >= the opponent's total.
     * Both teams gain a reroll in a tie.
     */
    public static boolean gainsExtraReroll(int ownTotal, int opponentTotal) {
        return ownTotal >= opponentTotal;
    }

    private KickoffEventCalc() {}
}
