package com.fumbbl.ffb;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/keyword_choice_mode.rs for {@link KeywordChoiceMode}.
 * The Rust for_name lookup has no Java analog (no KeywordChoiceModeFactory / forName), so that case is omitted.
 */
public class KeywordChoiceModeTest {

	@Test
	public void getDialogHeaderContainsPlayerName() {
		String header = KeywordChoiceMode.GETTING_EVEN.getDialogHeader("Griff");
		assertTrue(header.contains("Griff"));
	}

	@Test
	public void serdeRoundTrip() {
		assertEquals(KeywordChoiceMode.GETTING_EVEN, KeywordChoiceMode.valueOf(KeywordChoiceMode.GETTING_EVEN.name()));
	}

}
