package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.bb2020.InjuryDescription;

import org.junit.jupiter.api.Test;

import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_use_mortuary_assistants_parameter.rs for
 * {@link DialogUseMortuaryAssistantsParameter}.
 */
public class DialogUseMortuaryAssistantsParameterTest {

	@Test
	public void storesTeamIdAndInjuryDescriptions() {
		DialogUseMortuaryAssistantsParameter p = new DialogUseMortuaryAssistantsParameter("home",
			Arrays.asList(new InjuryDescription()), 0);
		assertEquals("home", p.getTeamId());
		assertEquals(1, p.getInjuryDescriptions().size());
	}

}
