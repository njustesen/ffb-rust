package com.fumbbl.ffb;

import com.fumbbl.ffb.factory.LeaderStateFactory;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/leader_state.rs for {@link LeaderState}.
 */
public class LeaderStateTest {

	private final LeaderStateFactory factory = new LeaderStateFactory();

	@Test
	public void forNameReturnsCorrectVariant() {
		assertEquals(LeaderState.NONE, factory.forName("none"));
		assertEquals(LeaderState.AVAILABLE, factory.forName("available"));
		assertEquals(LeaderState.USED, factory.forName("used"));
	}

	@Test
	public void forNameUnknownReturnsNone() {
		// The Rust for_name is case-sensitive so it rejects "AVAILABLE"; the Java
		// LeaderStateFactory.forName is case-insensitive (equalsIgnoreCase) and would
		// resolve "AVAILABLE" to AVAILABLE, so only the genuinely-unknown value is ported.
		assertNull(factory.forName("invalid"));
	}

	@Test
	public void getNameMatchesForNameInput() {
		for (LeaderState state : new LeaderState[]{LeaderState.NONE, LeaderState.AVAILABLE, LeaderState.USED}) {
			assertEquals(state, factory.forName(state.getName()));
		}
	}

}
