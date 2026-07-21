package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command.rs tests.
 * ClientCommand is abstract in Java; a minimal concrete subclass exercises the
 * base-class entropy handling and JSON base fields (super.toJsonValue/initFrom),
 * matching the Rust module which tests the base struct in isolation via a Dummy.
 */
public class ClientCommandTest {

	private static class TestClientCommand extends ClientCommand {
		public NetCommandId getId() {
			return NetCommandId.CLIENT_JOIN;
		}
	}

	@Test
	public void defaultNoEntropy() {
		assertFalse(new TestClientCommand().hasEntropy());
	}

	@Test
	public void entropyStored() {
		TestClientCommand cmd = new TestClientCommand();
		cmd.setEntropy((byte) 42);
		assertEquals((byte) 42, cmd.getEntropy());
	}

	@Test
	public void maxEntropyStored() {
		// Java entropy is a signed byte, so 255 round-trips as (byte) 255 == -1.
		// Rust models it as u8 (255). See discrepancy note in report.
		TestClientCommand cmd = new TestClientCommand();
		cmd.setEntropy((byte) 255);
		assertEquals((byte) 255, cmd.getEntropy());
	}

	@Test
	public void hasEntropyReflectsState() {
		assertFalse(new TestClientCommand().hasEntropy());
		TestClientCommand cmd = new TestClientCommand();
		cmd.setEntropy((byte) 9);
		assertTrue(cmd.hasEntropy());
	}

	@Test
	public void baseJsonFieldsIncludesNetCommandId() {
		JsonObject json = new TestClientCommand().toJsonValue();
		assertEquals("clientJoin", json.get("netCommandId").asString());
		assertNull(json.get("entropy"));
	}

	@Test
	public void baseJsonFieldsIncludesEntropyWhenPresent() {
		TestClientCommand cmd = new TestClientCommand();
		cmd.setEntropy((byte) 7);
		JsonObject json = cmd.toJsonValue();
		assertEquals(7, json.get("entropy").asInt());
	}

	@Test
	public void baseFromJsonRoundTrip() {
		TestClientCommand cmd = new TestClientCommand();
		cmd.setEntropy((byte) 3);
		JsonObject json = cmd.toJsonValue();
		TestClientCommand restored = new TestClientCommand();
		restored.initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 3, restored.getEntropy());
	}
}
