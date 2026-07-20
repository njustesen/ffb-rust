package com.fumbbl.ffb.server.model;

import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.FieldCoordinateBounds;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class FieldCoordinateTest {

    // ── FieldCoordinate ─────────────────────────────────────────────────────

    @Test
    void field_coordinate_get_x() {
        assertEquals(5, new FieldCoordinate(5, 3).getX());
    }

    @Test
    void field_coordinate_get_y() {
        assertEquals(3, new FieldCoordinate(5, 3).getY());
    }

    @Test
    void field_coordinate_home_box_is_box_coordinate() {
        assertTrue(new FieldCoordinate(FieldCoordinate.RSV_HOME_X, 1).isBoxCoordinate());
        assertTrue(new FieldCoordinate(FieldCoordinate.KO_HOME_X, 1).isBoxCoordinate());
    }

    @Test
    void field_coordinate_away_box_is_box_coordinate() {
        assertTrue(new FieldCoordinate(FieldCoordinate.RSV_AWAY_X, 1).isBoxCoordinate());
    }

    @Test
    void field_coordinate_pitch_coordinate_is_not_box() {
        assertFalse(new FieldCoordinate(5, 5).isBoxCoordinate());
    }

    @Test
    void field_coordinate_add_returns_correct_result() {
        FieldCoordinate result = new FieldCoordinate(5, 5).add(2, -1);
        assertEquals(7, result.getX());
        assertEquals(4, result.getY());
    }

    @Test
    void field_coordinate_adjacent_to_neighbor() {
        assertTrue(new FieldCoordinate(5, 5).isAdjacent(new FieldCoordinate(6, 5)));
        assertTrue(new FieldCoordinate(5, 5).isAdjacent(new FieldCoordinate(5, 6)));
    }

    @Test
    void field_coordinate_adjacent_to_diagonal() {
        assertTrue(new FieldCoordinate(5, 5).isAdjacent(new FieldCoordinate(6, 6)));
    }

    @Test
    void field_coordinate_not_adjacent_to_far_square() {
        assertFalse(new FieldCoordinate(1, 1).isAdjacent(new FieldCoordinate(5, 5)));
    }

    @Test
    void field_width_is_26() {
        assertEquals(26, FieldCoordinate.FIELD_WIDTH);
    }

    @Test
    void field_height_is_15() {
        assertEquals(15, FieldCoordinate.FIELD_HEIGHT);
    }

    @Test
    void distance_chebyshev() {
        FieldCoordinate a = new FieldCoordinate(0, 0);
        FieldCoordinate b = new FieldCoordinate(3, 1);
        assertEquals(3, a.distanceInSteps(b));
    }

    @Test
    void transform_mirrors_field() {
        FieldCoordinate c = new FieldCoordinate(10, 7);
        FieldCoordinate t = c.transform();
        assertEquals(FieldCoordinate.FIELD_WIDTH - 1 - 10, t.getX());
        assertEquals(7, t.getY());
        // Transform is its own inverse on-field
        assertEquals(c, t.transform());
    }

    @Test
    void transform_dugout() {
        FieldCoordinate homeRsv = new FieldCoordinate(FieldCoordinate.RSV_HOME_X, 3);
        FieldCoordinate awayRsv = homeRsv.transform();
        assertEquals(FieldCoordinate.RSV_AWAY_X, awayRsv.getX());
        assertEquals(FieldCoordinate.RSV_HOME_X, awayRsv.transform().getX());
    }

    @Test
    void direction_to() {
        FieldCoordinate origin = new FieldCoordinate(5, 5);
        assertEquals(Direction.NORTHEAST, origin.getDirection(new FieldCoordinate(6, 4)));
        assertNull(origin.getDirection(origin));
    }

    @Test
    void step_moves_by_direction_and_distance() {
        FieldCoordinate c = new FieldCoordinate(5, 5);
        FieldCoordinate east2 = c.move(Direction.EAST, 2);
        assertEquals(7, east2.getX());
        assertEquals(5, east2.getY());
        FieldCoordinate south1 = c.move(Direction.SOUTH, 1);
        assertEquals(5, south1.getX());
        assertEquals(6, south1.getY());
    }

    // ── FieldCoordinateBounds ───────────────────────────────────────────────

    @Test
    void field_coordinate_bounds_field_contains_center() {
        assertTrue(FieldCoordinateBounds.FIELD.isInBounds(new FieldCoordinate(13, 8)));
    }

    @Test
    void field_coordinate_bounds_has_top_left_corner() {
        assertNotNull(FieldCoordinateBounds.FIELD.getTopLeftCorner());
    }

    @Test
    void field_coordinate_bounds_has_bottom_right_corner() {
        assertNotNull(FieldCoordinateBounds.FIELD.getBottomRightCorner());
    }

    @Test
    void field_coordinate_bounds_field_width_is_26() {
        assertEquals(26, FieldCoordinateBounds.FIELD.width());
    }

    @Test
    void field_coordinate_bounds_field_height_is_15() {
        assertEquals(15, FieldCoordinateBounds.FIELD.height());
    }

    @Test
    void field_coordinate_bounds_endzone_home_in_bounds() {
        assertTrue(FieldCoordinateBounds.ENDZONE_HOME.isInBounds(new FieldCoordinate(0, 5)));
    }

    @Test
    void field_coordinate_bounds_endzone_away_in_bounds() {
        assertTrue(FieldCoordinateBounds.ENDZONE_AWAY.isInBounds(new FieldCoordinate(25, 5)));
    }

    @Test
    void bounds_in_bounds() {
        assertTrue(FieldCoordinateBounds.HALF_HOME.isInBounds(new FieldCoordinate(0, 0)));
        assertFalse(FieldCoordinateBounds.HALF_HOME.isInBounds(new FieldCoordinate(13, 0)));
    }

    @Test
    void bounds_size() {
        // FIELD is 26 wide x 15 tall = 390
        assertEquals(390, FieldCoordinateBounds.FIELD.size());
    }

    @Test
    void bounds_coordinates_count() {
        // x=12, y=4..10 -> 7 squares
        assertEquals(7, FieldCoordinateBounds.LOS_HOME.fieldCoordinates().length);
    }
}
