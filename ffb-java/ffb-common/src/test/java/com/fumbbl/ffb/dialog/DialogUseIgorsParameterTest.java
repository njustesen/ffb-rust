package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.bb2020.InjuryDescription;

import org.junit.jupiter.api.Test;

import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_use_igors_parameter.rs for
 * {@link DialogUseIgorsParameter}.
 */
public class DialogUseIgorsParameterTest {

	@Test
	public void storesInjuryDescriptions() {
		DialogUseIgorsParameter p = new DialogUseIgorsParameter(null, Arrays.asList(new InjuryDescription()), 0);
		assertEquals(1, p.getInjuryDescriptions().size());
	}

}
