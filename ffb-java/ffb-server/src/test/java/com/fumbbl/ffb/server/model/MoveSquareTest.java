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
    void pushback_square_home_choice_flips_on_transform() {
        PushbackSquare sq = new PushbackSquare(new FieldCoordinate(10, 7), Direction.EAST, true);
        PushbackSquare t = sq.transform();
        assertFalse(t.isHomeChoice());
        assertEquals(FieldCoordinate.FIELD_WIDTH - 1 - 10, t.getCoordinate().getX());
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
}
