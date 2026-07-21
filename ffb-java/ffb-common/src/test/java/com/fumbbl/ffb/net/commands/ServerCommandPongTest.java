package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.FactoryType.FactoryContext;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/server_command_pong.rs tests.
 */
public class ServerCommandPongTest {

	@Test
	public void timestampStored() {
		ServerCommandPong cmd = new ServerCommandPong(99999);
		assertEquals(99999L, cmd.getTimestamp());
	}

	@Test
	public void defaultZero() {
		ServerCommandPong cmd = new ServerCommandPong();
		assertEquals(0L, cmd.getTimestamp());
	}

	@Test
	public void getIdIsServerPong() {
		assertEquals(NetCommandId.SERVER_PONG, new ServerCommandPong(1).getId());
	}

	@Test
	public void contextIsApplication() {
		assertEquals(FactoryContext.APPLICATION, new ServerCommandPong(1).getContext());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndTimestamp() {
		ServerCommandPong cmd = new ServerCommandPong(555);
		JsonObject json = cmd.toJsonValue();
		assertEquals("serverPong", json.get("netCommandId").asString());
		assertEquals(555L, json.get("timestamp").asLong());
	}

	@Test
	public void toJsonValueNeverIncludesCommandNr() {
		ServerCommandPong cmd = new ServerCommandPong(1);
		JsonObject json = cmd.toJsonValue();
		assertNull(json.get("commandNr"));
	}

	@Test
	public void roundTripWithTimestamp() {
		ServerCommandPong cmd = new ServerCommandPong(9876);
		JsonObject json = cmd.toJsonValue();
		ServerCommandPong restored = new ServerCommandPong().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(9876L, restored.getTimestamp());
	}

	@Test
	public void roundTripWithDefaultZeroTimestamp() {
		ServerCommandPong cmd = new ServerCommandPong();
		JsonObject json = cmd.toJsonValue();
		ServerCommandPong restored = new ServerCommandPong().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(0L, restored.getTimestamp());
	}
}
