package com.fumbbl.ffb.server.model;

import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.model.BlockRoll;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class BlockRollTest {

    private static final PlayerState STANDING = new PlayerState(PlayerState.STANDING);

    @Test
    void block_roll_needs_selection_when_index_negative() {
        BlockRoll br = new BlockRoll("p1", STANDING, 1);
        assertTrue(br.needsSelection());
    }

    @Test
    void block_roll_does_not_need_selection_when_index_zero() {
        BlockRoll br = new BlockRoll("p1", STANDING, 1);
        br.setSelectedIndex(0);
        assertFalse(br.needsSelection());
    }

    @Test
    void block_roll_target_id_set_on_construction() {
        BlockRoll br = new BlockRoll("player42", STANDING, 1);
        assertEquals("player42", br.getTargetId());
    }

    @Test
    void block_roll_dauntless_false_by_default() {
        BlockRoll br = new BlockRoll("p1", STANDING, 1);
        assertFalse(br.isSuccessFulDauntless());
    }

    @Test
    void block_roll_reroll_sources_empty_initially() {
        BlockRoll br = new BlockRoll("p1", STANDING, 1);
        assertTrue(br.getReRollSources().isEmpty());
    }
}
