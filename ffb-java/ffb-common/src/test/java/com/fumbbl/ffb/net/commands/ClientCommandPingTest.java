package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_ping.rs tests.
 * The Rust {@code debug_format_nonempty} test is Rust-specific (exercises the Debug impl)
 * and is not portable to Java; it is skipped.
 */
public class ClientCommandPingTest {

	@Test
	public void timestampStored() {
		ClientCommandPing cmd = new ClientCommandPing(12345);
		assertEquals(12345, cmd.getTimestamp());
	}

	@Test
	public void defaultZero() {
		ClientCommandPing cmd = new ClientCommandPing();
		assertEquals(0, cmd.getTimestamp());
	}

	@Test
	public void getIdIsClientPing() {
		assertEquals(NetCommandId.CLIENT_PING, new ClientCommandPing().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndTimestamp() {
		ClientCommandPing cmd = new ClientCommandPing(999);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientPing", json.get("netCommandId").asString());
		assertEquals(999, json.get("timestamp").asLong());
	}

	@Test
	public void roundTripWithData() {
		ClientCommandPing cmd = new ClientCommandPing(42);
		JsonObject json = cmd.toJsonValue();
		ClientCommandPing restored = new ClientCommandPing().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(42, restored.getTimestamp());
	}

	@Test
	public void roundTripDefault() {
		ClientCommandPing cmd = new ClientCommandPing();
		JsonObject json = cmd.toJsonValue();
		ClientCommandPing restored = new ClientCommandPing().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(0, restored.getTimestamp());
	}
}
