package com.fumbbl.ffb.server.model;

import com.fumbbl.ffb.ReRollOptions;
import com.fumbbl.ffb.ReRollProperty;
import org.junit.jupiter.api.Test;

import java.util.Arrays;
import java.util.Collections;
import java.util.List;

import static org.junit.jupiter.api.Assertions.*;

class ReRollOptionsTest {

    @Test
    void reroll_options_can_actually_reroll_when_trr_property() {
        ReRollOptions options = new ReRollOptions(Arrays.asList(ReRollProperty.TRR), null);
        assertTrue(options.canActuallyReRoll());
    }

    @Test
    void reroll_options_cannot_reroll_when_empty() {
        ReRollOptions options = new ReRollOptions(Collections.emptyList(), null);
        assertFalse(options.canActuallyReRoll());
    }

    @Test
    void reroll_options_property_list_not_empty_when_set() {
        List<ReRollProperty> props = Arrays.asList(ReRollProperty.TRR, ReRollProperty.LONER);
        ReRollOptions options = new ReRollOptions(props, null);
        assertFalse(options.getProperties().isEmpty());
    }

    @Test
    void reroll_options_loner_cannot_actually_reroll_alone() {
        ReRollOptions options = new ReRollOptions(Arrays.asList(ReRollProperty.LONER), null);
        assertFalse(options.canActuallyReRoll());
    }
}
