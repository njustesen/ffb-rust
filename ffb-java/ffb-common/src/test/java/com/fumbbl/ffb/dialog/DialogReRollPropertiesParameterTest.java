package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.ReRollProperty;

import org.junit.jupiter.api.Test;

import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_re_roll_properties_parameter.rs for
 * {@link DialogReRollPropertiesParameter}.
 */
public class DialogReRollPropertiesParameterTest {

	@Test
	public void hasPropertyTrueWhenPresent() {
		DialogReRollPropertiesParameter p = new DialogReRollPropertiesParameter(null, null, 0,
			Arrays.asList(ReRollProperty.TRR), false, null, null, null, null, null);
		assertTrue(p.hasProperty(ReRollProperty.TRR));
		assertFalse(p.hasProperty(ReRollProperty.LONER));
	}

	@Test
	public void hasPropertyFalseWhenAbsent() {
		DialogReRollPropertiesParameter p = new DialogReRollPropertiesParameter();
		assertFalse(p.hasProperty(ReRollProperty.TRR));
	}

}
