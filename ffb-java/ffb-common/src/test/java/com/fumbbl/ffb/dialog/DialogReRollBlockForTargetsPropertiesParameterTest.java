package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.model.BlockRollProperties;

import org.junit.jupiter.api.Test;

import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_re_roll_block_for_targets_properties_parameter.rs
 * for {@link DialogReRollBlockForTargetsPropertiesParameter}.
 */
public class DialogReRollBlockForTargetsPropertiesParameterTest {

	@Test
	public void blockRollsStoredCorrectly() {
		BlockRollProperties roll = new BlockRollProperties();
		DialogReRollBlockForTargetsPropertiesParameter p =
			new DialogReRollBlockForTargetsPropertiesParameter(null, Arrays.asList(roll));
		assertEquals(1, p.getBlockRolls().size());
		assertEquals(roll, p.getBlockRolls().get(0));
	}

}
