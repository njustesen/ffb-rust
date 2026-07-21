package com.fumbbl.ffb;

import com.fumbbl.ffb.factory.CatchScatterThrowInModeFactory;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/catch_scatter_throw_in_mode.rs for {@link CatchScatterThrowInMode}.
 */
public class CatchScatterThrowInModeTest {

	private final CatchScatterThrowInModeFactory factory = new CatchScatterThrowInModeFactory();

	@Test
	public void forNameRoundTrip() {
		assertEquals(CatchScatterThrowInMode.CATCH_HAND_OFF, factory.forName("catchHandOff"));
	}

	@Test
	public void forNameUnknownReturnsNone() {
		assertNull(factory.forName("unknown"));
	}

}
