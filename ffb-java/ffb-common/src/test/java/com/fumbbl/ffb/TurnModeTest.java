package com.fumbbl.ffb;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/turn_mode.rs for {@link TurnMode}.
 */
public class TurnModeTest {

	@Test
	public void forNameRoundTrip() {
		assertEquals(TurnMode.REGULAR, TurnMode.forName("regular"));
		assertEquals(TurnMode.BLITZ, TurnMode.forName("blitz"));
	}

	@Test
	public void forNameUnknownReturnsNone() {
		assertNull(TurnMode.forName("notATurnMode"));
	}

}
