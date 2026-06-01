package com.fumbbl.ffb.server.model;

import com.fumbbl.ffb.CardEffect;
import com.fumbbl.ffb.CardTarget;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.EnumSource;

import static org.junit.jupiter.api.Assertions.*;

class CardEnumTest {

    // ── CardEffect ──────────────────────────────────────────────────────────

    @Test
    void card_effect_count_is_five() {
        assertEquals(5, CardEffect.values().length);
    }

    @ParameterizedTest
    @EnumSource(CardEffect.class)
    void card_effect_all_have_non_null_name(CardEffect e) {
        assertNotNull(e.getName());
        assertFalse(e.getName().isEmpty());
    }

    @Test
    void card_effect_distracted_name() {
        assertEquals("Distracted", CardEffect.DISTRACTED.getName());
    }

    @Test
    void card_effect_sedative_name() {
        assertEquals("Sedative", CardEffect.SEDATIVE.getName());
    }

    @Test
    void card_effect_poisoned_name() {
        assertEquals("Poisoned", CardEffect.POISONED.getName());
    }

    @Test
    void card_effect_distracted_has_skills() {
        assertFalse(CardEffect.DISTRACTED.skills().isEmpty());
    }

    @Test
    void card_effect_poisoned_has_no_skills() {
        assertTrue(CardEffect.POISONED.skills().isEmpty());
    }

    @Test
    void card_effect_mad_cap_mushroom_has_two_skills() {
        assertEquals(2, CardEffect.MAD_CAP_MUSHROOM_POTION.skills().size());
    }

    // ── CardTarget ──────────────────────────────────────────────────────────

    @Test
    void card_target_count_is_four() {
        assertEquals(4, CardTarget.values().length);
    }

    @ParameterizedTest
    @EnumSource(CardTarget.class)
    void card_target_all_have_non_null_name(CardTarget t) {
        assertNotNull(t.getName());
        assertFalse(t.getName().isEmpty());
    }

    @Test
    void card_target_turn_id_is_one() {
        assertEquals(1, CardTarget.TURN.getId());
    }

    @Test
    void card_target_own_player_id_is_two() {
        assertEquals(2, CardTarget.OWN_PLAYER.getId());
    }

    @Test
    void card_target_turn_is_not_played_on_player() {
        assertFalse(CardTarget.TURN.isPlayedOnPlayer());
    }

    @Test
    void card_target_own_player_is_played_on_player() {
        assertTrue(CardTarget.OWN_PLAYER.isPlayedOnPlayer());
    }

    @Test
    void card_target_any_player_is_played_on_player() {
        assertTrue(CardTarget.ANY_PLAYER.isPlayedOnPlayer());
    }

    @Test
    void card_target_from_id() {
        assertEquals(CardTarget.TURN, CardTarget.fromId(1));
        assertEquals(CardTarget.ANY_PLAYER, CardTarget.fromId(4));
    }

    @Test
    void card_target_from_id_unknown_returns_null() {
        assertNull(CardTarget.fromId(99));
    }
}
