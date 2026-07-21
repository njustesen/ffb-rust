package com.fumbbl.ffb;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/status_type.rs for {@link StatusType}.
 */
public class StatusTypeTest {

	@Test
	public void serdeRoundTrip() {
		assertEquals(StatusType.WAITING, StatusType.valueOf(StatusType.WAITING.name()));
	}

	@Test
	public void serdeRoundTripRef() {
		assertEquals(StatusType.REF, StatusType.valueOf(StatusType.REF.name()));
	}

}
