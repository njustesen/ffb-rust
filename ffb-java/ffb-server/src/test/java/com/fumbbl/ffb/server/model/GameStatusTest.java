package com.fumbbl.ffb.server.model;

import com.fumbbl.ffb.GameStatus;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.EnumSource;

import static org.junit.jupiter.api.Assertions.*;

class GameStatusTest {

    @Test
    void game_status_count_is_nine() {
        assertEquals(9, GameStatus.values().length);
    }

    @ParameterizedTest
    @EnumSource(GameStatus.class)
    void game_status_all_have_non_null_name(GameStatus s) {
        assertNotNull(s.getName());
        assertFalse(s.getName().isEmpty());
    }

    @ParameterizedTest
    @EnumSource(GameStatus.class)
    void game_status_all_have_non_null_type_string(GameStatus s) {
        assertNotNull(s.getTypeString());
        assertFalse(s.getTypeString().isEmpty());
    }

    @Test
    void game_status_active_name() {
        assertEquals("active", GameStatus.ACTIVE.getName());
    }

    @Test
    void game_status_active_type_string() {
        assertEquals("A", GameStatus.ACTIVE.getTypeString());
    }

    @Test
    void game_status_finished_name() {
        assertEquals("finished", GameStatus.FINISHED.getName());
    }

    @Test
    void game_status_scheduled_type_string() {
        assertEquals("O", GameStatus.SCHEDULED.getTypeString());
    }

    @Test
    void game_status_loading_name() {
        assertEquals("loading", GameStatus.LOADING.getName());
    }

    @Test
    void game_status_type_strings_are_unique() {
        long unique = java.util.Arrays.stream(GameStatus.values())
            .map(GameStatus::getTypeString)
            .distinct().count();
        assertEquals(GameStatus.values().length, unique);
    }
}
