package com.fumbbl.ffb;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/model/breathe_fire_result.rs for
 * {@link BreatheFireResult}.
 */
public class BreatheFireResultTest {

	@Test
	public void serdeRoundTrip() {
		for (BreatheFireResult v : new BreatheFireResult[] { BreatheFireResult.FAILURE, BreatheFireResult.NO_EFFECT,
			BreatheFireResult.PRONE, BreatheFireResult.KNOCK_DOWN }) {
			assertEquals(v, BreatheFireResult.valueOf(v.name()));
		}
	}
}
