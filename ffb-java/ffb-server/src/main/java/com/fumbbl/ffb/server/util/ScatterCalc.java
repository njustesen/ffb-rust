package com.fumbbl.ffb.server.util;

import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.FieldCoordinate;

public final class ScatterCalc {

    /**
     * Map a D8 roll (1–8) to a scatter direction.
     * Mirrors DirectionFactory.forRoll(): 1=North, 2=Northeast, ..., 8=Northwest.
     */
    public static Direction directionForRoll(int roll) {
        switch (roll) {
        case 1: return Direction.NORTH;
        case 2: return Direction.NORTHEAST;
        case 3: return Direction.EAST;
        case 4: return Direction.SOUTHEAST;
        case 5: return Direction.SOUTH;
        case 6: return Direction.SOUTHWEST;
        case 7: return Direction.WEST;
        case 8: return Direction.NORTHWEST;
        default: return null;
        }
    }

    /**
     * Compute the coordinate after scattering from {@code start} in {@code direction} for {@code distance} squares.
     * Does not clamp or validate board bounds.
     */
    public static FieldCoordinate scatterCoordinate(FieldCoordinate start, Direction direction, int distance) {
        switch (direction) {
        case NORTH:     return start.add(0, -distance);
        case NORTHEAST: return start.add(distance, -distance);
        case EAST:      return start.add(distance, 0);
        case SOUTHEAST: return start.add(distance, distance);
        case SOUTH:     return start.add(0, distance);
        case SOUTHWEST: return start.add(-distance, distance);
        case WEST:      return start.add(-distance, 0);
        case NORTHWEST: return start.add(-distance, -distance);
        default:        throw new IllegalArgumentException("Unknown direction: " + direction);
        }
    }

    private ScatterCalc() {}
}
