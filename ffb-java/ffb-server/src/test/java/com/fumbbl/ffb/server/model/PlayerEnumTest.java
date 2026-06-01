package com.fumbbl.ffb.server.model;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.PlayerType;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.EnumSource;

import static org.junit.jupiter.api.Assertions.*;

class PlayerEnumTest {

    // ── PlayerGender ────────────────────────────────────────────────────────

    @Test
    void player_gender_count_is_four() {
        assertEquals(4, PlayerGender.values().length);
    }

    @ParameterizedTest
    @EnumSource(PlayerGender.class)
    void player_gender_all_have_non_null_name(PlayerGender g) {
        assertNotNull(g.getName());
    }

    @Test
    void player_gender_male_nominative_is_he() {
        assertEquals("he", PlayerGender.MALE.getNominative());
    }

    @Test
    void player_gender_female_nominative_is_she() {
        assertEquals("she", PlayerGender.FEMALE.getNominative());
    }

    @Test
    void player_gender_nonbinary_nominative_is_they() {
        assertEquals("they", PlayerGender.NONBINARY.getNominative());
    }

    @Test
    void player_gender_neutral_nominative_is_it() {
        assertEquals("it", PlayerGender.NEUTRAL.getNominative());
    }

    @Test
    void player_gender_male_type_string_is_M() {
        assertEquals("M", PlayerGender.MALE.getTypeString());
    }

    @Test
    void player_gender_from_ordinal_one_is_male() {
        assertEquals(PlayerGender.MALE, PlayerGender.fromOrdinal(1));
    }

    @Test
    void player_gender_from_ordinal_two_is_female() {
        assertEquals(PlayerGender.FEMALE, PlayerGender.fromOrdinal(2));
    }

    @Test
    void player_gender_from_ordinal_default_is_neutral() {
        assertEquals(PlayerGender.NEUTRAL, PlayerGender.fromOrdinal(99));
    }

    // ── PlayerType ──────────────────────────────────────────────────────────

    @Test
    void player_type_count_is_nine() {
        assertEquals(9, PlayerType.values().length);
    }

    @ParameterizedTest
    @EnumSource(PlayerType.class)
    void player_type_all_have_non_null_name(PlayerType t) {
        assertNotNull(t.getName());
        assertFalse(t.getName().isEmpty());
    }

    @Test
    void player_type_regular_name() {
        assertEquals("Regular", PlayerType.REGULAR.getName());
    }

    @Test
    void player_type_star_name() {
        assertEquals("Star", PlayerType.STAR.getName());
    }

    @Test
    void player_type_big_guy_name() {
        assertEquals("Big Guy", PlayerType.BIG_GUY.getName());
    }
}
