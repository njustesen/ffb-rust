package com.fumbbl.ffb.server.util;

import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.RulesCollection.Rules;

/**
 * Pure throw-in mechanics: direction and distance from D6 rolls.
 *
 * Board bounds: x=0 is home endzone, x=25 is away endzone; y=0 is upper sideline, y=14 is lower sideline.
 */
public final class ThrowInCalc {

    /**
     * Throw-in distance from two D6 results.
     * BB2020 adds 1; all other editions sum the two dice directly.
     */
    public static int throwInDistance(int die1, int die2, Rules rules) {
        int base = die1 + die2;
        return (rules == Rules.BB2020) ? base + 1 : base;
    }

    /**
     * Whether the coordinate is a corner square (BB2025 only).
     * Corners exist at the intersections of both endzones and both sidelines.
     */
    public static boolean isCornerSquare(int x, int y) {
        return (x < 1 || x > 24) && (y < 1 || y > 13);
    }

    /**
     * Throw-in direction from a D6 roll (1–6) based on which edge the ball left from.
     * Returns one of three directions: the two diagonals flanking the inward direction, or straight in.
     *
     * Coordinate conventions:
     * - x < 1  → home endzone, ball goes EAST (inward)
     * - x > 24 → away endzone, ball goes WEST (inward)
     * - y > 13 → lower sideline, ball goes NORTH (inward)
     * - y < 1  → upper sideline, ball goes SOUTH (inward)
     */
    public static Direction throwInDirectionForRoll(int x, int y, int roll) {
        if (x < 1)  return throwInDirectionFromTemplate(Direction.EAST,  roll);
        if (x > 24) return throwInDirectionFromTemplate(Direction.WEST,  roll);
        if (y > 13) return throwInDirectionFromTemplate(Direction.NORTH, roll);
        if (y < 1)  return throwInDirectionFromTemplate(Direction.SOUTH, roll);
        throw new IllegalArgumentException("Coordinate (" + x + "," + y + ") is not on the board edge.");
    }

    /**
     * Throw-in direction from a D3 roll (1–3) for BB2025 corner squares.
     * The corner direction identifies which corner (e.g. NORTHWEST = x<1, y<1).
     */
    public static Direction cornerThrowInDirectionForRoll(Direction cornerDirection, int roll) {
        switch (cornerDirection) {
        case NORTHWEST:
            switch (roll) { case 1: return Direction.EAST;  case 2: return Direction.SOUTHEAST; default: return Direction.SOUTH; }
        case NORTHEAST:
            switch (roll) { case 1: return Direction.SOUTH; case 2: return Direction.SOUTHWEST;  default: return Direction.WEST; }
        case SOUTHWEST:
            switch (roll) { case 1: return Direction.NORTH; case 2: return Direction.NORTHEAST;  default: return Direction.EAST; }
        case SOUTHEAST:
            switch (roll) { case 1: return Direction.WEST;  case 2: return Direction.NORTHWEST;  default: return Direction.NORTH; }
        default:
            throw new IllegalArgumentException("Not a corner direction: " + cornerDirection);
        }
    }

    /**
     * Which corner direction applies to the given corner coordinate (BB2025).
     */
    public static Direction cornerDirection(int x, int y) {
        boolean west  = x < 1;
        boolean north = y < 1;
        if (west  && north) return Direction.NORTHWEST;
        if (!west && north) return Direction.NORTHEAST;
        if (west)           return Direction.SOUTHWEST;
        return Direction.SOUTHEAST;
    }

    // Mirrors ThrowInMechanic.interpretThrowInDirectionRoll(Direction, int)
    private static Direction throwInDirectionFromTemplate(Direction template, int roll) {
        switch (template) {
        case EAST:
            switch (roll) { case 1: case 2: return Direction.NORTHEAST; case 3: case 4: return Direction.EAST; default: return Direction.SOUTHEAST; }
        case WEST:
            switch (roll) { case 1: case 2: return Direction.SOUTHWEST; case 3: case 4: return Direction.WEST; default: return Direction.NORTHWEST; }
        case NORTH:
            switch (roll) { case 1: case 2: return Direction.NORTHWEST; case 3: case 4: return Direction.NORTH; default: return Direction.NORTHEAST; }
        case SOUTH:
            switch (roll) { case 1: case 2: return Direction.SOUTHEAST; case 3: case 4: return Direction.SOUTH; default: return Direction.SOUTHWEST; }
        default:
            throw new IllegalArgumentException("Not a cardinal direction template: " + template);
        }
    }

    private ThrowInCalc() {}
}
