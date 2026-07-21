package com.fumbbl.ffb.dialog;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/util_dialog_parameter.rs for
 * {@link UtilDialogParameter}.
 */
public class UtilDialogParameterTest {

	@Test
	public void validateMatchingIdPasses() {
		DialogReRollParameter param = new DialogReRollParameter();
		UtilDialogParameter.validateDialogId(param, DialogId.RE_ROLL);
	}

	@Test
	public void validateMismatchedIdPanics() {
		DialogBlockRollParameter param = new DialogBlockRollParameter();
		IllegalStateException ex = assertThrows(IllegalStateException.class,
			() -> UtilDialogParameter.validateDialogId(param, DialogId.RE_ROLL));
		assertTrue(ex.getMessage().contains("Wrong dialog id"));
	}

	@Test
	public void validateBlockRollWithBlockRollIdPasses() {
		DialogBlockRollParameter param = new DialogBlockRollParameter();
		UtilDialogParameter.validateDialogId(param, DialogId.BLOCK_ROLL);
	}

	@Test
	public void validateReRollWithBlockRollIdPanics() {
		DialogReRollParameter param = new DialogReRollParameter();
		IllegalStateException ex = assertThrows(IllegalStateException.class,
			() -> UtilDialogParameter.validateDialogId(param, DialogId.BLOCK_ROLL));
		assertTrue(ex.getMessage().contains("Wrong dialog id"));
	}

	@Test
	public void validateBlockRollPasses() {
		DialogBlockRollParameter param = new DialogBlockRollParameter();
		UtilDialogParameter.validateDialogId(param, DialogId.BLOCK_ROLL);
	}

}
