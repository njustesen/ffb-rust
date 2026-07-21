package com.fumbbl.ffb.dialog;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_journeymen_parameter.rs for
 * {@link DialogJourneymenParameter}.
 */
public class DialogJourneymenParameterTest {

	@Test
	public void addPositionIdAppendsNonemptyStrings() {
		DialogJourneymenParameter p = new DialogJourneymenParameter(null, 0, new String[] { "pos1", "" });
		assertArrayEquals(new String[] { "pos1" }, p.getPositionIds());
	}

}
