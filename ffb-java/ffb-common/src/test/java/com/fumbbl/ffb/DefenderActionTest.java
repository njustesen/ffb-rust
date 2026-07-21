package com.fumbbl.ffb;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/defender_action.rs for {@link DefenderAction}.
 */
public class DefenderActionTest {

	@Test
	public void fromIdRoundTrips() {
		assertEquals(DefenderAction.DUMP_OFF, DefenderAction.fromId(1));
		assertNull(DefenderAction.fromId(0));
	}

	@Test
	public void serdeRoundTrip() {
		assertEquals(DefenderAction.DUMP_OFF, DefenderAction.valueOf(DefenderAction.DUMP_OFF.name()));
	}

	@Test
	public void fromIdNegativeReturnsNone() {
		assertNull(DefenderAction.fromId(-1));
		assertNull(DefenderAction.fromId(99));
	}

}
