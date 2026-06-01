package com.fumbbl.ffb.server.util;

import com.fumbbl.ffb.PassingDistance;

/**
 * Pure passing-distance calculation extracted from PassMechanic.
 *
 * The distance between two squares is determined by a 14×14 lookup table
 * indexed by [deltaY][deltaX] (both ≥ 0). Out-of-range deltas (≥ 14) return null.
 * The [0][0] cell is also null (cannot pass to own square).
 */
public final class PassingDistanceCalc {

    private PassingDistanceCalc() {}

    /**
     * Passing distance table, same as BB2020/BB2025 PassMechanic.
     * Indexed: table[deltaY][deltaX].
     * null entries mean the delta is out of range or invalid.
     */
    private static final PassingDistance[][] TABLE = buildTable();

    private static PassingDistance[][] buildTable() {
        // Row strings from BB2020 PassMechanic.throwingRangeTable():
        // Q=QuickPass, S=ShortPass, L=LongPass, B=LongBomb, T/space=null
        String[] rows = {
            "T Q Q Q S S S L L L L B B B",
            "Q Q Q Q S S S L L L L B B B",
            "Q Q Q S S S S L L L L B B B",
            "Q Q S S S S S L L L B B B  ",
            "S S S S S S L L L L B B B  ",
            "S S S S S L L L L B B B    ",
            "S S S S L L L L L B B B    ",
            "L L L L L L L L B B B      ",
            "L L L L L L L B B B B      ",
            "L L L L L B B B B B        ",
            "L L L B B B B B B          ",
            "B B B B B B B              ",
            "B B B B B                  ",
            "B B B                      "
        };
        PassingDistance[][] table = new PassingDistance[14][14];
        for (int dy = 0; dy < 14; dy++) {
            String row = rows[dy];
            for (int dx = 0; dx < 14; dx++) {
                int idx = dx * 2;
                if (idx >= row.length()) {
                    table[dy][dx] = null;
                    continue;
                }
                char c = row.charAt(idx);
                table[dy][dx] = fromChar(c);
            }
        }
        return table;
    }

    private static PassingDistance fromChar(char c) {
        switch (c) {
            case 'Q': return PassingDistance.QUICK_PASS;
            case 'S': return PassingDistance.SHORT_PASS;
            case 'L': return PassingDistance.LONG_PASS;
            case 'B': return PassingDistance.LONG_BOMB;
            default:  return null; // T, space, or unknown
        }
    }

    /**
     * Returns the passing distance for a throw with the given absolute coordinate deltas.
     * Returns null if the distance is out of range (too far or same square).
     *
     * @param deltaX absolute x-difference between target and thrower
     * @param deltaY absolute y-difference between target and thrower
     * @return passing distance category, or null if out of range
     */
    public static PassingDistance forDeltas(int deltaX, int deltaY) {
        if (deltaX < 0 || deltaY < 0 || deltaX >= 14 || deltaY >= 14) {
            return null;
        }
        return TABLE[deltaY][deltaX];
    }

    /**
     * Returns the passing distance for a throw from (fromX, fromY) to (toX, toY).
     * Returns null if the distance is out of range or from==to.
     */
    public static PassingDistance forCoordinates(int fromX, int fromY, int toX, int toY) {
        return forDeltas(Math.abs(toX - fromX), Math.abs(toY - fromY));
    }
}
