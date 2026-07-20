package com.fumbbl.ffb.server.model;

import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.factory.DirectionFactory;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.EnumSource;

import static org.junit.jupiter.api.Assertions.*;

class DirectionTest {

    @Test
    void direction_count_is_eight() {
        assertEquals(8, Direction.values().length);
    }

    @ParameterizedTest
    @EnumSource(Direction.class)
    void direction_all_have_non_null_name(Direction d) {
        assertNotNull(d.getName());
        assertFalse(d.getName().isEmpty());
    }

    @Test
    void direction_north_name() {
        assertEquals("North", Direction.NORTH.getName());
    }

    @Test
    void direction_south_name() {
        assertEquals("South", Direction.SOUTH.getName());
    }

    @Test
    void direction_east_name() {
        assertEquals("East", Direction.EAST.getName());
    }

    @Test
    void direction_west_name() {
        assertEquals("West", Direction.WEST.getName());
    }

    @Test
    void direction_northeast_name() {
        assertEquals("Northeast", Direction.NORTHEAST.getName());
    }

    @Test
    void direction_all_values_found_by_name() {
        for (Direction d : Direction.values()) {
            assertEquals(d, Direction.forName(d.getName()));
        }
    }

    @Test
    void direction_for_name_case_insensitive() {
        assertEquals(Direction.NORTH, Direction.forName("north"));
        assertEquals(Direction.SOUTH, Direction.forName("SOUTH"));
    }

    @Test
    void direction_for_name_unknown_returns_null() {
        assertNull(Direction.forName("unknown"));
    }

    @Test
    void direction_transform_north_to_north() {
        assertEquals(Direction.NORTH, Direction.NORTH.transform());
    }

    @Test
    void direction_transform_east_to_west() {
        assertEquals(Direction.WEST, Direction.EAST.transform());
    }

    @Test
    void direction_transform_northeast_to_northwest() {
        assertEquals(Direction.NORTHWEST, Direction.NORTHEAST.transform());
    }

    @Test
    void direction_transform_southeast_to_southwest() {
        assertEquals(Direction.SOUTHWEST, Direction.SOUTHEAST.transform());
    }

    @Test
    void direction_transform_south_to_south() {
        assertEquals(Direction.SOUTH, Direction.SOUTH.transform());
    }

    @Test
    void direction_transform_is_involution() {
        for (Direction d : Direction.values()) {
            assertEquals(d, d.transform().transform());
        }
    }

    @Test
    void direction_for_roll_all_eight_faces() {
        DirectionFactory factory = new DirectionFactory();
        assertEquals(Direction.NORTH, factory.forRoll(1));
        assertEquals(Direction.NORTHEAST, factory.forRoll(2));
        assertEquals(Direction.EAST, factory.forRoll(3));
        assertEquals(Direction.SOUTHEAST, factory.forRoll(4));
        assertEquals(Direction.SOUTH, factory.forRoll(5));
        assertEquals(Direction.SOUTHWEST, factory.forRoll(6));
        assertEquals(Direction.WEST, factory.forRoll(7));
        assertEquals(Direction.NORTHWEST, factory.forRoll(8));
    }

    @Test
    void direction_for_roll_out_of_range_returns_null() {
        DirectionFactory factory = new DirectionFactory();
        assertNull(factory.forRoll(0));
        assertNull(factory.forRoll(9));
    }
}
