package com.fumbbl.ffb.server.model;

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
}
