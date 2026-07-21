package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.ReRollProperty;

import org.junit.jupiter.api.Test;

import java.util.Arrays;
import java.util.Collections;
import java.util.HashMap;
import java.util.Map;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_block_roll_properties_parameter.rs for
 * {@link DialogBlockRollPropertiesParameter}.
 */
public class DialogBlockRollPropertiesParameterTest {

	@Test
	public void hasPropertyTrueWhenPresent() {
		DialogBlockRollPropertiesParameter p = new DialogBlockRollPropertiesParameter(null, 0, null,
			Arrays.asList(ReRollProperty.TRR), null);
		assertTrue(p.hasProperty(ReRollProperty.TRR));
		assertFalse(p.hasProperty(ReRollProperty.LONER));
	}

	@Test
	public void hasActualReRollTrueForTrr() {
		DialogBlockRollPropertiesParameter p = new DialogBlockRollPropertiesParameter(null, 0, null,
			Arrays.asList(ReRollProperty.TRR), null);
		assertTrue(p.hasActualReRoll());
	}

	@Test
	public void hasActualReRollFalseForLonerOnly() {
		DialogBlockRollPropertiesParameter p = new DialogBlockRollPropertiesParameter(null, 0, null,
			Arrays.asList(ReRollProperty.LONER), null);
		assertFalse(p.hasActualReRoll());
	}

	@Test
	public void hasActualReRollTrueWhenRrActionToSourceNonempty() {
		Map<String, String> rrActionToSource = new HashMap<>();
		rrActionToSource.put("block", "team");
		DialogBlockRollPropertiesParameter p = new DialogBlockRollPropertiesParameter(null, 0, null,
			Collections.emptyList(), rrActionToSource);
		assertTrue(p.hasActualReRoll());
	}

	@Test
	public void hasActualReRollFalseWhenEmpty() {
		DialogBlockRollPropertiesParameter p = new DialogBlockRollPropertiesParameter();
		assertFalse(p.hasActualReRoll());
	}

}
