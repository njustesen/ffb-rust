package com.fumbbl.ffb.server.model;

import com.fumbbl.ffb.BoxType;
import com.fumbbl.ffb.SendToBoxReason;
import com.fumbbl.ffb.TeamStatus;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.EnumSource;

import static org.junit.jupiter.api.Assertions.*;

class TeamEnumTest {

    // ── BoxType ─────────────────────────────────────────────────────────────

    @Test
    void box_type_count_is_two() {
        assertEquals(2, BoxType.values().length);
    }

    @Test
    void box_type_reserves_id_is_one() {
        assertEquals(1, BoxType.RESERVES.getId());
    }

    @Test
    void box_type_out_id_is_two() {
        assertEquals(2, BoxType.OUT.getId());
    }

    @Test
    void box_type_reserves_name() {
        assertEquals("reserves", BoxType.RESERVES.getName());
    }

    @Test
    void box_type_from_id_one_is_reserves() {
        assertEquals(BoxType.RESERVES, BoxType.fromId(1));
    }

    @Test
    void box_type_from_id_two_is_out() {
        assertEquals(BoxType.OUT, BoxType.fromId(2));
    }

    @Test
    void box_type_from_id_unknown_returns_null() {
        assertNull(BoxType.fromId(99));
    }

    @Test
    void box_type_from_name() {
        assertEquals(BoxType.RESERVES, BoxType.fromName("reserves"));
        assertEquals(BoxType.OUT, BoxType.fromName("out"));
    }

    // ── SendToBoxReason ─────────────────────────────────────────────────────

    @ParameterizedTest
    @EnumSource(SendToBoxReason.class)
    void send_to_box_reason_all_have_non_null_name(SendToBoxReason r) {
        assertNotNull(r.getName());
        assertFalse(r.getName().isEmpty());
    }

    @ParameterizedTest
    @EnumSource(SendToBoxReason.class)
    void send_to_box_reason_all_have_non_null_reason(SendToBoxReason r) {
        assertNotNull(r.getReason());
        assertFalse(r.getReason().isEmpty());
    }

    @Test
    void send_to_box_reason_mng_name() {
        assertEquals("mng", SendToBoxReason.MNG.getName());
    }

    @Test
    void send_to_box_reason_foul_ban_name() {
        assertEquals("foulBan", SendToBoxReason.FOUL_BAN.getName());
    }

    @Test
    void send_to_box_reason_blocked_reason() {
        assertEquals("was blocked", SendToBoxReason.BLOCKED.getReason());
    }

    @Test
    void send_to_box_reason_at_least_thirty_seven_variants() {
        assertTrue(SendToBoxReason.values().length >= 37);
    }

    // ── TeamStatus ──────────────────────────────────────────────────────────

    @Test
    void team_status_count_is_seven() {
        assertEquals(7, TeamStatus.values().length);
    }

    @ParameterizedTest
    @EnumSource(TeamStatus.class)
    void team_status_all_have_non_null_name(TeamStatus s) {
        assertNotNull(s.getName());
        assertFalse(s.getName().isEmpty());
    }

    @Test
    void team_status_new_id_is_zero() {
        assertEquals(0, TeamStatus.NEW.getId());
    }

    @Test
    void team_status_active_id_is_one() {
        assertEquals(1, TeamStatus.ACTIVE.getId());
    }

    @Test
    void team_status_active_name() {
        assertEquals("Active", TeamStatus.ACTIVE.getName());
    }

    @Test
    void team_status_skill_rolls_pending_id_is_six() {
        assertEquals(6, TeamStatus.SKILL_ROLLS_PENDING.getId());
    }
}
