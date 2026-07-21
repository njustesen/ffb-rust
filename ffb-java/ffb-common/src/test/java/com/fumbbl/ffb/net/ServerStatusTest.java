package com.fumbbl.ffb.net;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/server_status.rs tests.
 * The Rust serde_round_trip test has no Java analogue (ServerStatus is not JSON
 * serialized directly in Java); it is intentionally not ported.
 */
public class ServerStatusTest {

	@Test
	public void nameMatchesJava() {
		assertEquals("Unknown Coach", ServerStatus.ERROR_UNKNOWN_COACH.getName());
		assertEquals("Replay Unavailable", ServerStatus.REPLAY_UNAVAILABLE.getName());
	}

	@Test
	public void messageNonEmptyForAllVariants() {
		for (ServerStatus v : ServerStatus.values()) {
			assertFalse(v.getMessage().isEmpty(), v + " message must not be empty");
		}
	}
}
