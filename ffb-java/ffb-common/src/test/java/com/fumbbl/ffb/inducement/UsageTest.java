package com.fumbbl.ffb.inducement;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/inducement/usage.rs for {@link Usage}.
 */
public class UsageTest {

	@Test
	public void serdeRoundTrip() {
		Usage u = Usage.ADD_CHEERLEADER;
		Usage back = Usage.valueOf(u.name());
		assertEquals(u, back);
	}
}
