package com.fumbbl.ffb.dialog;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_setup_error_parameter.rs for
 * {@link DialogSetupErrorParameter}.
 */
public class DialogSetupErrorParameterTest {

	@Test
	public void addSetupErrorFiltersEmpty() {
		DialogSetupErrorParameter p = new DialogSetupErrorParameter(null, new String[] { "err1", "" });
		assertEquals(1, p.getSetupErrors().length);
	}

}
