package com.fumbbl.ffb.dialog;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_argue_the_call_parameter.rs for
 * {@link DialogArgueTheCallParameter}.
 */
public class DialogArgueTheCallParameterTest {

	@Test
	public void addPlayerIdAppendsNonemptyStrings() {
		DialogArgueTheCallParameter p = new DialogArgueTheCallParameter();
		p.addPlayerId("p1");
		p.addPlayerId("p2");
		assertArrayEquals(new String[] { "p1", "p2" }, p.getPlayerIds());
	}

	@Test
	public void addPlayerIdIgnoresEmptyString() {
		DialogArgueTheCallParameter p = new DialogArgueTheCallParameter();
		p.addPlayerId("");
		assertEquals(0, p.getPlayerIds().length);
	}

}
