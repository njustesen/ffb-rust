package com.fumbbl.ffb.dialog;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_team_setup_parameter.rs for
 * {@link DialogTeamSetupParameter}.
 */
public class DialogTeamSetupParameterTest {

	@Test
	public void addSetupNameFiltersEmpty() {
		DialogTeamSetupParameter p = new DialogTeamSetupParameter(false, new String[] { "MySetup", "" });
		assertEquals(1, p.getSetupNames().length);
		assertEquals("MySetup", p.getSetupNames()[0]);
	}

}
