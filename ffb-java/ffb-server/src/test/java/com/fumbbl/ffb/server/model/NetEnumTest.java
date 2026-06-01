package com.fumbbl.ffb.server.model;

import com.fumbbl.ffb.LeaderState;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.ServerStatus;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.EnumSource;

import static org.junit.jupiter.api.Assertions.*;

class NetEnumTest {

    // ── NetCommandId ────────────────────────────────────────────────────────

    @ParameterizedTest
    @EnumSource(NetCommandId.class)
    void net_command_id_all_have_non_null_name(NetCommandId id) {
        assertNotNull(id.getName());
        assertFalse(id.getName().isEmpty());
    }

    @Test
    void net_command_id_at_least_seventy_variants() {
        assertTrue(NetCommandId.values().length >= 70);
    }

    @Test
    void net_command_id_client_join_name() {
        assertEquals("clientJoin", NetCommandId.CLIENT_JOIN.getName());
    }

    @Test
    void net_command_id_server_game_state_name() {
        assertEquals("serverGameState", NetCommandId.SERVER_GAME_STATE.getName());
    }

    @Test
    void net_command_id_client_end_turn_name() {
        assertEquals("clientEndTurn", NetCommandId.CLIENT_END_TURN.getName());
    }

    @Test
    void net_command_id_names_are_unique() {
        long unique = java.util.Arrays.stream(NetCommandId.values())
            .map(NetCommandId::getName)
            .distinct().count();
        assertEquals(NetCommandId.values().length, unique);
    }

    // ── ServerStatus ────────────────────────────────────────────────────────

    @Test
    void server_status_count_is_eight() {
        assertEquals(8, ServerStatus.values().length);
    }

    @ParameterizedTest
    @EnumSource(ServerStatus.class)
    void server_status_all_have_non_null_name(ServerStatus s) {
        assertNotNull(s.getName());
        assertFalse(s.getName().isEmpty());
    }

    @ParameterizedTest
    @EnumSource(ServerStatus.class)
    void server_status_all_have_non_null_message(ServerStatus s) {
        assertNotNull(s.getMessage());
        assertFalse(s.getMessage().isEmpty());
    }

    @Test
    void server_status_error_wrong_password_name() {
        assertEquals("Wrong Password", ServerStatus.ERROR_WRONG_PASSWORD.getName());
    }

    @Test
    void server_status_fumbbl_error_name() {
        assertEquals("Fumbbl Error", ServerStatus.FUMBBL_ERROR.getName());
    }

    // ── LeaderState ─────────────────────────────────────────────────────────

    @Test
    void leader_state_count_is_three() {
        assertEquals(3, LeaderState.values().length);
    }

    @ParameterizedTest
    @EnumSource(LeaderState.class)
    void leader_state_all_have_non_null_name(LeaderState s) {
        assertNotNull(s.getName());
        assertFalse(s.getName().isEmpty());
    }

    @Test
    void leader_state_none_name() {
        assertEquals("none", LeaderState.NONE.getName());
    }

    @Test
    void leader_state_available_name() {
        assertEquals("available", LeaderState.AVAILABLE.getName());
    }

    @Test
    void leader_state_used_name() {
        assertEquals("used", LeaderState.USED.getName());
    }
}
