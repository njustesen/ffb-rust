package com.fumbbl.ffb;

import com.fumbbl.ffb.factory.PushbackModeFactory;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/pushback_mode.rs for {@link PushbackMode}.
 */
public class PushbackModeTest {

	private final PushbackModeFactory factory = new PushbackModeFactory();

	@Test
	public void forNameRoundTrips() {
		assertEquals(PushbackMode.SIDE_STEP, factory.forName("sideStep"));
		assertNull(factory.forName("invalid"));
	}

	@Test
	public void forNameCoversAllVariants() {
		assertEquals(PushbackMode.REGULAR, factory.forName("regular"));
		assertEquals(PushbackMode.SIDE_STEP, factory.forName("sideStep"));
		assertEquals(PushbackMode.GRAB, factory.forName("grab"));
		assertNull(factory.forName("unknown"));
	}

}
