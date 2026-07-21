package com.fumbbl.ffb.net.commands;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/i_command_with_acting_player.rs tests.
 * ICommandWithActingPlayer is a marker interface exposing getActingPlayerId();
 * a small implementor exercises the set/unset cases (Rust Option None -> null).
 */
public class ICommandWithActingPlayerTest {

	private static class TestCmd implements ICommandWithActingPlayer {
		private final String playerId;

		TestCmd(String playerId) {
			this.playerId = playerId;
		}

		public String getActingPlayerId() {
			return playerId;
		}
	}

	@Test
	public void returnsSomeWhenSet() {
		assertEquals("p1", new TestCmd("p1").getActingPlayerId());
	}

	@Test
	public void returnsNoneWhenUnset() {
		assertNull(new TestCmd(null).getActingPlayerId());
	}
}
