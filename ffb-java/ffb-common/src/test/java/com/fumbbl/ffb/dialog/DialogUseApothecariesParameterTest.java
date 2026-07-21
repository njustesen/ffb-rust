package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.bb2020.InjuryDescription;

import org.junit.jupiter.api.Test;

import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_use_apothecaries_parameter.rs for
 * {@link DialogUseApothecariesParameter}.
 */
public class DialogUseApothecariesParameterTest {

	@Test
	public void storesInjuryDescriptions() {
		DialogUseApothecariesParameter p = new DialogUseApothecariesParameter("away",
			Arrays.asList(new InjuryDescription()));
		assertEquals(1, p.getInjuryDescriptions().size());
		assertEquals("away", p.getTeamId());
	}

}
