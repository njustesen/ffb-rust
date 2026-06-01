package com.fumbbl.ffb.server.model;

import com.fumbbl.ffb.ApothecaryMode;
import com.fumbbl.ffb.ApothecaryStatus;
import com.fumbbl.ffb.ApothecaryType;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.EnumSource;

import static org.junit.jupiter.api.Assertions.*;

class ApothecaryEnumTest {

    // ── ApothecaryMode ──────────────────────────────────────────────────────

    @Test
    void apothecary_mode_count_is_fifteen() {
        assertEquals(15, ApothecaryMode.values().length);
    }

    @ParameterizedTest
    @EnumSource(ApothecaryMode.class)
    void apothecary_mode_all_have_non_null_name(ApothecaryMode m) {
        assertNotNull(m.getName());
        assertFalse(m.getName().isEmpty());
    }

    @Test
    void apothecary_mode_home_name() {
        assertEquals("home", ApothecaryMode.HOME.getName());
    }

    @Test
    void apothecary_mode_away_name() {
        assertEquals("away", ApothecaryMode.AWAY.getName());
    }

    @Test
    void apothecary_mode_defender_name() {
        assertEquals("defender", ApothecaryMode.DEFENDER.getName());
    }

    @Test
    void apothecary_mode_attacker_name() {
        assertEquals("attacker", ApothecaryMode.ATTACKER.getName());
    }

    // ── ApothecaryStatus ────────────────────────────────────────────────────

    @Test
    void apothecary_status_count_is_eleven() {
        assertEquals(11, ApothecaryStatus.values().length);
    }

    @ParameterizedTest
    @EnumSource(ApothecaryStatus.class)
    void apothecary_status_all_have_non_null_name(ApothecaryStatus s) {
        assertNotNull(s.getName());
        assertFalse(s.getName().isEmpty());
    }

    @Test
    void apothecary_status_use_apothecary_name() {
        assertEquals("useApothecary", ApothecaryStatus.USE_APOTHECARY.getName());
    }

    @Test
    void apothecary_status_no_apothecary_name() {
        assertEquals("noApothecary", ApothecaryStatus.NO_APOTHECARY.getName());
    }

    // ── ApothecaryType ──────────────────────────────────────────────────────

    @Test
    void apothecary_type_count_is_three() {
        assertEquals(3, ApothecaryType.values().length);
    }

    @ParameterizedTest
    @EnumSource(ApothecaryType.class)
    void apothecary_type_all_have_non_null_name(ApothecaryType t) {
        assertNotNull(t.getName());
        assertFalse(t.getName().isEmpty());
    }

    @Test
    void apothecary_type_team_name() {
        assertEquals("Team Apothecary", ApothecaryType.TEAM.getName());
    }

    @Test
    void apothecary_type_wandering_name() {
        assertEquals("Wandering Apothecary", ApothecaryType.WANDERING.getName());
    }

    @Test
    void apothecary_type_plague_name() {
        assertEquals("Plague Doctor", ApothecaryType.PLAGUE.getName());
    }
}
