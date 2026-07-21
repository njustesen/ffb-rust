package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.model.BlockRollProperties;

import org.junit.jupiter.api.Test;

import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_opponent_block_selection_properties_parameter.rs
 * for {@link DialogOpponentBlockSelectionPropertiesParameter}.
 */
public class DialogOpponentBlockSelectionPropertiesParameterTest {

	@Test
	public void blockRollsStoredCorrectly() {
		BlockRollProperties roll = new BlockRollProperties();
		DialogOpponentBlockSelectionPropertiesParameter p =
			new DialogOpponentBlockSelectionPropertiesParameter(null, Arrays.asList(roll));
		assertEquals(1, p.getBlockRolls().size());
		assertEquals(roll, p.getBlockRolls().get(0));
	}

}
