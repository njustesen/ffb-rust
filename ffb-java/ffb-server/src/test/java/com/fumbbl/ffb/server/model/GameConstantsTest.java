package com.fumbbl.ffb.server.model;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.FieldCoordinateBounds;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class GameConstantsTest {

    @Test
    void field_total_size_is_390() {
        assertEquals(390, FieldCoordinate.FIELD_WIDTH * FieldCoordinate.FIELD_HEIGHT);
    }

    @Test
    void field_bounds_field_size_is_390() {
        assertEquals(390, FieldCoordinateBounds.FIELD.width() * FieldCoordinateBounds.FIELD.height());
    }

    @Test
    void endzone_home_width_is_one() {
        assertEquals(1, FieldCoordinateBounds.ENDZONE_HOME.width());
    }

    @Test
    void endzone_away_width_is_one() {
        assertEquals(1, FieldCoordinateBounds.ENDZONE_AWAY.width());
    }
}
