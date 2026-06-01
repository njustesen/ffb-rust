package com.fumbbl.ffb.server.model;

import com.fumbbl.ffb.TurnMode;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.EnumSource;

import static org.junit.jupiter.api.Assertions.*;

class TurnModeTest {

    @ParameterizedTest
    @EnumSource(TurnMode.class)
    void turn_mode_all_have_non_null_name(TurnMode t) {
        assertNotNull(t.getName());
        assertFalse(t.getName().isEmpty());
    }

    @Test
    void turn_mode_regular_is_basic_mode() {
        assertTrue(TurnMode.REGULAR.isBasicMode());
    }

    @Test
    void turn_mode_blitz_is_basic_mode() {
        assertTrue(TurnMode.BLITZ.isBasicMode());
    }

    @Test
    void turn_mode_setup_is_not_basic_mode() {
        assertFalse(TurnMode.SETUP.isBasicMode());
    }

    @Test
    void turn_mode_regular_checks_for_active_players() {
        assertTrue(TurnMode.REGULAR.isCheckForActivePlayers());
    }

    @Test
    void turn_mode_setup_does_not_check_for_active_players() {
        assertFalse(TurnMode.SETUP.isCheckForActivePlayers());
    }

    @Test
    void turn_mode_bomb_home_is_bomb_turn() {
        assertTrue(TurnMode.BOMB_HOME.isBombTurn());
        assertTrue(TurnMode.BOMB_AWAY.isBombTurn());
        assertTrue(TurnMode.BOMB_HOME_BLITZ.isBombTurn());
        assertTrue(TurnMode.BOMB_AWAY_BLITZ.isBombTurn());
    }

    @Test
    void turn_mode_regular_is_not_bomb_turn() {
        assertFalse(TurnMode.REGULAR.isBombTurn());
    }

    @Test
    void turn_mode_dump_off_does_not_allow_end_player_action() {
        assertFalse(TurnMode.DUMP_OFF.allowEndPlayerAction());
    }

    @Test
    void turn_mode_regular_allows_end_player_action() {
        assertTrue(TurnMode.REGULAR.allowEndPlayerAction());
    }

    @Test
    void turn_mode_trickster_forces_dice_decoration_update() {
        assertTrue(TurnMode.TRICKSTER.forceDiceDecorationUpdate());
    }

    @Test
    void turn_mode_regular_does_not_force_dice_decoration_update() {
        assertFalse(TurnMode.REGULAR.forceDiceDecorationUpdate());
    }

    @Test
    void turn_mode_for_name_regular() {
        assertEquals(TurnMode.REGULAR, TurnMode.forName("regular"));
    }

    @Test
    void turn_mode_for_name_case_insensitive() {
        assertEquals(TurnMode.KICKOFF, TurnMode.forName("KICKOFF"));
    }

    @Test
    void turn_mode_kickoff_return_does_not_check_negaTraits() {
        assertFalse(TurnMode.KICKOFF_RETURN.checkNegatraits());
    }

    @Test
    void turn_mode_regular_checks_negaTraits() {
        assertTrue(TurnMode.REGULAR.checkNegatraits());
    }
}
