package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.model.BlockRoll;

import org.junit.jupiter.api.Test;

import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_opponent_block_selection_parameter.rs for
 * {@link DialogOpponentBlockSelectionParameter}.
 */
public class DialogOpponentBlockSelectionParameterTest {

	@Test
	public void blockRollsStoredCorrectly() {
		BlockRoll roll = new BlockRoll();
		DialogOpponentBlockSelectionParameter p = new DialogOpponentBlockSelectionParameter(null, Arrays.asList(roll));
		assertEquals(1, p.getBlockRolls().size());
		assertEquals(roll, p.getBlockRolls().get(0));
	}

}
