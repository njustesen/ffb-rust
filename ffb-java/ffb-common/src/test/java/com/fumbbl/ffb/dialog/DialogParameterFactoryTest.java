package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.IDialogParameter;
import com.fumbbl.ffb.factory.DialogIdFactory;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_parameter_factory.rs for
 * {@link DialogParameterFactory}.
 */
public class DialogParameterFactoryTest {

	private final DialogParameterFactory factory = new DialogParameterFactory();

	@Test
	public void knownDialogIdReturnsSome() {
		IDialogParameter parameter = factory.createDialogParameter(DialogId.BLOCK_ROLL);
		assertNotNull(parameter);
		assertTrue(parameter instanceof DialogBlockRollParameter);
	}

	@Test
	public void unknownDialogIdReturnsNone() {
		DialogId unknown = new DialogIdFactory().forName("UNKNOWN_DIALOG");
		assertNull(factory.createDialogParameter(unknown));
	}

	@Test
	public void puntToCrowdIsLastVariant() {
		IDialogParameter parameter = factory.createDialogParameter(DialogId.PUNT_TO_CROWD);
		assertNotNull(parameter);
		assertTrue(parameter instanceof DialogPuntToCrowdParameter);
	}

}
