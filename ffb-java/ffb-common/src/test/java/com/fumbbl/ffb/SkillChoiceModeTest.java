package com.fumbbl.ffb;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/skill_choice_mode.rs for {@link SkillChoiceMode}.
 */
public class SkillChoiceModeTest {

	@Test
	public void getDialogHeaderIncludesPlayerName() {
		assertTrue(SkillChoiceMode.INTENSIVE_TRAINING.getDialogHeader("Bob").contains("Bob"));
	}

	@Test
	public void statusMessageShared() {
		// Both variants share the same status message.
		assertEquals(SkillChoiceMode.INTENSIVE_TRAINING.getStatusMessage(),
			SkillChoiceMode.WISDOM_OF_THE_WHITE_DWARF.getStatusMessage());
	}

}
