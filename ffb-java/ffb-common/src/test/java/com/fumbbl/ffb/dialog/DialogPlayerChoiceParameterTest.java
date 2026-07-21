package com.fumbbl.ffb.dialog;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_player_choice_parameter.rs for
 * {@link DialogPlayerChoiceParameter}.
 */
public class DialogPlayerChoiceParameterTest {

	@Test
	public void addPlayerIdAppendsNonempty() {
		DialogPlayerChoiceParameter p = new DialogPlayerChoiceParameter();
		p.addPlayerId("p1");
		p.addPlayerId("");
		assertArrayEquals(new String[] { "p1" }, p.getPlayerIds());
	}

	@Test
	public void addDescriptionAppendsNonempty() {
		DialogPlayerChoiceParameter p = new DialogPlayerChoiceParameter();
		p.addDescription("Select a target");
		p.addDescription("");
		assertArrayEquals(new String[] { "Select a target" }, p.getDescriptions());
	}

}
