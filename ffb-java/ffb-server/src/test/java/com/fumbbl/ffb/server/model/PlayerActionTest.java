package com.fumbbl.ffb.server.model;

import com.fumbbl.ffb.PlayerAction;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.EnumSource;

import static org.junit.jupiter.api.Assertions.*;

class PlayerActionTest {

    @Test
    void player_action_count_at_least_fifty() {
        assertTrue(PlayerAction.values().length >= 50);
    }

    @ParameterizedTest
    @EnumSource(PlayerAction.class)
    void player_action_all_have_non_null_name(PlayerAction a) {
        assertNotNull(a.getName());
        assertFalse(a.getName().isEmpty());
    }

    @Test
    void move_is_moving() {
        assertTrue(PlayerAction.MOVE.isMoving());
    }

    @Test
    void blitz_move_is_moving() {
        assertTrue(PlayerAction.BLITZ_MOVE.isMoving());
    }

    @Test
    void pass_move_is_moving() {
        assertTrue(PlayerAction.PASS_MOVE.isMoving());
    }

    @Test
    void block_is_not_moving() {
        assertFalse(PlayerAction.BLOCK.isMoving());
    }

    @Test
    void pass_is_not_moving() {
        assertFalse(PlayerAction.PASS.isMoving());
    }

    @Test
    void pass_is_passing() {
        assertTrue(PlayerAction.PASS.isPassing());
    }

    @Test
    void dump_off_is_passing() {
        assertTrue(PlayerAction.DUMP_OFF.isPassing());
    }

    @Test
    void hand_over_is_passing() {
        assertTrue(PlayerAction.HAND_OVER.isPassing());
    }

    @Test
    void move_is_not_passing() {
        assertFalse(PlayerAction.MOVE.isPassing());
    }

    @Test
    void blitz_is_blitzing() {
        assertTrue(PlayerAction.BLITZ.isBlitzing());
    }

    @Test
    void blitz_move_is_blitzing() {
        assertTrue(PlayerAction.BLITZ_MOVE.isBlitzing());
    }

    @Test
    void move_is_not_blitzing() {
        assertFalse(PlayerAction.MOVE.isBlitzing());
    }

    @Test
    void block_is_block_action() {
        assertTrue(PlayerAction.BLOCK.isBlockAction());
    }

    @Test
    void chainsaw_is_block_action() {
        assertTrue(PlayerAction.CHAINSAW.isBlockAction());
    }

    @Test
    void stab_is_block_action() {
        assertTrue(PlayerAction.STAB.isBlockAction());
    }

    @Test
    void move_is_not_block_action() {
        assertFalse(PlayerAction.MOVE.isBlockAction());
    }

    @Test
    void move_type_is_one() {
        assertEquals(1, PlayerAction.MOVE.getType());
    }

    @Test
    void block_type_is_two() {
        assertEquals(2, PlayerAction.BLOCK.getType());
    }

    @Test
    void blitz_type_is_three() {
        assertEquals(3, PlayerAction.BLITZ.getType());
    }

    @Test
    void pass_type_is_seven() {
        assertEquals(7, PlayerAction.PASS.getType());
    }

    @Test
    void foul_type_is_nine() {
        assertEquals(9, PlayerAction.FOUL.getType());
    }

    @Test
    void stand_up_type_is_eleven() {
        assertEquals(11, PlayerAction.STAND_UP.getType());
    }

    @Test
    void gaze_is_gaze() {
        assertTrue(PlayerAction.GAZE.isGaze());
        assertTrue(PlayerAction.GAZE_MOVE.isGaze());
        assertTrue(PlayerAction.GAZE_SELECT.isGaze());
    }

    @Test
    void move_is_not_gaze() {
        assertFalse(PlayerAction.MOVE.isGaze());
    }

    @Test
    void throw_bomb_is_bomb() {
        assertTrue(PlayerAction.THROW_BOMB.isBomb());
    }

    @Test
    void move_is_not_bomb() {
        assertFalse(PlayerAction.MOVE.isBomb());
    }

    @Test
    void stand_up_and_stand_up_blitz_are_standing_up() {
        assertTrue(PlayerAction.STAND_UP.isStandingUp());
        assertTrue(PlayerAction.STAND_UP_BLITZ.isStandingUp());
        assertFalse(PlayerAction.MOVE.isStandingUp());
    }
}
