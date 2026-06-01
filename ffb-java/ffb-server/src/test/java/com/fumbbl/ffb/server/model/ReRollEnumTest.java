package com.fumbbl.ffb.server.model;

import com.fumbbl.ffb.ReRollProperty;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.EnumSource;

import static org.junit.jupiter.api.Assertions.*;

class ReRollEnumTest {

    @Test
    void reroll_property_count_is_seven() {
        assertEquals(7, ReRollProperty.values().length);
    }

    @ParameterizedTest
    @EnumSource(ReRollProperty.class)
    void reroll_property_all_have_non_null_name(ReRollProperty p) {
        assertNotNull(p.getName());
        assertFalse(p.getName().isEmpty());
    }

    @Test
    void reroll_property_trr_name() {
        assertEquals("TRR", ReRollProperty.TRR.getName());
    }

    @Test
    void reroll_property_trr_is_actual_reroll() {
        assertTrue(ReRollProperty.TRR.isActualReRoll());
    }

    @Test
    void reroll_property_mascot_is_actual_reroll() {
        assertTrue(ReRollProperty.MASCOT.isActualReRoll());
    }

    @Test
    void reroll_property_pro_is_actual_reroll() {
        assertTrue(ReRollProperty.PRO.isActualReRoll());
    }

    @Test
    void reroll_property_brilliant_coaching_is_not_actual_reroll() {
        assertFalse(ReRollProperty.BRILLIANT_COACHING.isActualReRoll());
    }

    @Test
    void reroll_property_loner_is_not_actual_reroll() {
        assertFalse(ReRollProperty.LONER.isActualReRoll());
    }

    @Test
    void reroll_property_name_matches_enum_name() {
        for (ReRollProperty p : ReRollProperty.values()) {
            assertEquals(p.name(), p.getName());
        }
    }
}
