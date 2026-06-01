package com.fumbbl.ffb.server.model;

import com.fumbbl.ffb.ClientStateId;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.EnumSource;

import static org.junit.jupiter.api.Assertions.*;

class ClientStateIdTest {

    @ParameterizedTest
    @EnumSource(ClientStateId.class)
    void client_state_id_all_have_non_null_name(ClientStateId id) {
        assertNotNull(id.getName());
        assertFalse(id.getName().isEmpty());
    }

    @Test
    void client_state_id_login_name() {
        assertEquals("login", ClientStateId.LOGIN.getName());
    }

    @Test
    void client_state_id_select_player_name() {
        assertEquals("selectPlayer", ClientStateId.SELECT_PLAYER.getName());
    }

    @Test
    void client_state_id_block_name() {
        assertEquals("block", ClientStateId.BLOCK.getName());
    }

    @Test
    void client_state_id_setup_name() {
        assertEquals("setup", ClientStateId.SETUP.getName());
    }

    @Test
    void client_state_id_move_name() {
        assertEquals("move", ClientStateId.MOVE.getName());
    }

    @Test
    void client_state_id_names_are_unique() {
        long unique = java.util.Arrays.stream(ClientStateId.values())
            .map(ClientStateId::getName)
            .distinct().count();
        assertEquals(ClientStateId.values().length, unique);
    }

    @Test
    void client_state_id_toString_equals_name() {
        for (ClientStateId id : ClientStateId.values()) {
            assertEquals(id.getName(), id.toString());
        }
    }

    @Test
    void client_state_id_at_least_forty_variants() {
        assertTrue(ClientStateId.values().length >= 40);
    }
}
