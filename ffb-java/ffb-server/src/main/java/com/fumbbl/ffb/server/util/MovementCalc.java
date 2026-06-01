package com.fumbbl.ffb.server.util;

/**
 * Pure movement range calculations extracted from UtilPlayer.hasMoveLeft() and related methods.
 *
 * Movement works as follows:
 *   totalSquares = MA + gfiSquares
 *   goingForIt   = currentMove >= MA
 *   standingUp costs STAND_UP_COST (3) squares off MA; if MA ≤ 3, must roll to stand up
 *   GFI always requires a 2+ roll (minimum 2, capped at 2 even with negative modifiers)
 */
public final class MovementCalc {

    /** Going For It adds this many extra squares beyond MA by default. */
    public static final int STANDARD_GFI_SQUARES = 2;

    /** Standing up from prone costs this many movement squares. */
    public static final int STAND_UP_COST = 3;

    /** Minimum roll required for any GFI attempt (all editions). */
    public static final int GFI_MINIMUM_ROLL = 2;

    private MovementCalc() {}

    /**
     * Maximum squares a player may move this action.
     *
     * @param ma          base movement allowance (with any temporary modifiers already applied)
     * @param gfiSquares  extra squares from Going For It (0 if not GFI, 2 normally, 3+ with skills)
     */
    public static int maxMovement(int ma, int gfiSquares) {
        return ma + gfiSquares;
    }

    /**
     * Whether the player's next square requires a GFI roll.
     *
     * GFI triggers as soon as currentMove equals or exceeds MA.
     * (The GFI squares themselves are permitted but require the roll.)
     */
    public static boolean isNextMoveGoingForIt(int currentMove, int ma) {
        return currentMove >= ma;
    }

    /**
     * Whether a prone player with the given MA must roll to stand up.
     *
     * Standing up requires a 4+ roll if MA ≤ 3 (cost of standing = 3 squares ≥ MA).
     */
    public static boolean mustRollToStandUp(int ma) {
        return ma <= STAND_UP_COST;
    }

    /**
     * Whether the player has movement remaining (including potential GFI).
     *
     * @param currentMove  squares already moved this action
     * @param ma           base movement allowance
     * @param gfiSquares   extra squares available via GFI (0 if not using GFI)
     */
    public static boolean hasMoveLeft(int currentMove, int ma, int gfiSquares) {
        return currentMove < ma + gfiSquares;
    }

    /**
     * GFI squares for a standard action: base 2, optionally +1 if player has an Extra GFI skill.
     */
    public static int gfiSquares(boolean hasExtraGfi) {
        return STANDARD_GFI_SQUARES + (hasExtraGfi ? 1 : 0);
    }
}
