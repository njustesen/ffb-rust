package com.fumbbl.ffb.server.model;

import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.MoveSquare;
import com.fumbbl.ffb.PushbackSquare;
import com.fumbbl.ffb.RangeRuler;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class MoveSquareTest {

    @Test
    void move_square_is_dodging_when_dodge_roll_positive() {
        MoveSquare sq = new MoveSquare(new FieldCoordinate(5, 5), 3, 0);
        assertTrue(sq.isDodging());
    }

    @Test
    void move_square_not_dodging_when_roll_zero() {
        MoveSquare sq = new MoveSquare(new FieldCoordinate(5, 5), 0, 0);
        assertFalse(sq.isDodging());
    }

    @Test
    void move_square_is_going_for_it_when_gfi_roll_positive() {
        MoveSquare sq = new MoveSquare(new FieldCoordinate(5, 5), 0, 2);
        assertTrue(sq.isGoingForIt());
        assertFalse(sq.isDodging());
    }

    @Test
    void transform_mirrors_coordinate() {
        MoveSquare sq = new MoveSquare(new FieldCoordinate(10, 7), 3, 0);
        MoveSquare t = sq.transform();
        assertEquals(25 - 10, t.getCoordinate().getX());
        assertEquals(3, t.getMinimumRollDodge());
    }

    @Test
    void transform_preserves_rolls() {
        MoveSquare sq = new MoveSquare(new FieldCoordinate(8, 3), 4, 2);
        MoveSquare t = sq.transform();
        // coordinate should be mirrored but rolls must be unchanged
        assertEquals(4, t.getMinimumRollDodge());
        assertEquals(2, t.getMinimumRollGoForIt());
        // y coordinate is untouched by transform
        assertEquals(3, t.getCoordinate().getY());
    }

    @Test
    void pushback_square_home_choice_flips_on_transform() {
        PushbackSquare sq = new PushbackSquare(new FieldCoordinate(10, 7), Direction.EAST, true);
        PushbackSquare t = sq.transform();
        assertFalse(t.isHomeChoice());
        assertEquals(FieldCoordinate.FIELD_WIDTH - 1 - 10, t.getCoordinate().getX());
    }

    @Test
    void pushback_square_transform_mirrors_direction() {
        PushbackSquare sq = new PushbackSquare(new FieldCoordinate(10, 7), Direction.EAST, true);
        PushbackSquare t = sq.transform();
        assertEquals(Direction.WEST, t.getDirection());
    }

    @Test
    void new_starts_unselected_and_unlocked() {
        PushbackSquare sq = new PushbackSquare(new FieldCoordinate(3, 3), Direction.SOUTH, true);
        assertFalse(sq.isSelected());
        assertFalse(sq.isLocked());
        assertTrue(sq.isHomeChoice());
    }

    @Test
    void transform_double_inverts_home_choice() {
        PushbackSquare sq = new PushbackSquare(new FieldCoordinate(10, 7), Direction.EAST, true);
        PushbackSquare t = sq.transform().transform();
        assertEquals(sq.isHomeChoice(), t.isHomeChoice());
    }

    @Test
    void range_ruler_minimum_roll_dash_for_zero() {
        RangeRuler r = new RangeRuler("p1", new FieldCoordinate(5, 5), 0, false);
        assertEquals("--", r.getMinimumRoll());
    }

    @Test
    void range_ruler_minimum_roll_3plus_for_roll_3() {
        RangeRuler r = new RangeRuler("p1", new FieldCoordinate(5, 5), 3, false);
        assertEquals("3+", r.getMinimumRoll());
    }

    @Test
    void display_strings() {
        assertEquals("--", new RangeRuler("p1", null, 0, false).getMinimumRoll());
        assertEquals("", new RangeRuler("p1", null, -1, false).getMinimumRoll());
        assertEquals("3+", new RangeRuler("p1", null, 3, false).getMinimumRoll());
        assertEquals("6", new RangeRuler("p1", null, 6, false).getMinimumRoll());
    }

    @Test
    void display_strings_all_in_range_values() {
        assertEquals("2+", new RangeRuler("p1", null, 2, false).getMinimumRoll());
        assertEquals("4+", new RangeRuler("p1", null, 4, false).getMinimumRoll());
        assertEquals("5+", new RangeRuler("p1", null, 5, false).getMinimumRoll());
        // Java formats any positive roll below 6 as "<roll>+", including 1
        assertEquals("1+", new RangeRuler("p1", null, 1, false).getMinimumRoll());
    }

    @Test
    void transform_mirrors_target() {
        RangeRuler r = new RangeRuler("p1", new FieldCoordinate(10, 7), 3, false);
        RangeRuler t = r.transform();
        assertEquals(25 - 10, t.getTargetCoordinate().getX());
        assertEquals("3+", t.getMinimumRoll());
    }

    @Test
    void transform_with_no_target_coordinate() {
        RangeRuler r = new RangeRuler("p2", null, 5, true);
        RangeRuler t = r.transform();
        // null target should remain null after transform
        assertNull(t.getTargetCoordinate());
        // other fields preserved
        assertEquals("p2", t.getThrowerId());
        assertEquals("5+", t.getMinimumRoll());
        assertTrue(t.isThrowTeamMate());
    }
}
