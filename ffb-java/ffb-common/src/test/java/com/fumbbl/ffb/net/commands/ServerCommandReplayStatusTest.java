package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/server_command_replay_status.rs tests.
 */
public class ServerCommandReplayStatusTest {

	@Test
	public void allFieldsStored() {
		ServerCommandReplayStatus cmd = new ServerCommandReplayStatus(42, 2, true, true, false);
		assertEquals(42, cmd.getCommandNr());
		assertEquals(2, cmd.getSpeed());
		assertTrue(cmd.isRunning());
		assertTrue(cmd.isForward());
		assertFalse(cmd.isSkip());
	}

	@Test
	public void defaultIsStopped() {
		ServerCommandReplayStatus cmd = new ServerCommandReplayStatus();
		assertFalse(cmd.isRunning());
		assertEquals(0, cmd.getCommandNr());
	}

	@Test
	public void getIdIsServerReplayStatus() {
		assertEquals(NetCommandId.SERVER_REPLAY_STATUS, new ServerCommandReplayStatus().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndFields() {
		ServerCommandReplayStatus cmd = new ServerCommandReplayStatus(42, 2, true, true, false);
		JsonObject json = cmd.toJsonValue();
		assertEquals("serverReplayStatus", json.get("netCommandId").asString());
		assertEquals(42, json.get("commandNr").asInt());
		assertEquals(2, json.get("speed").asInt());
		assertTrue(json.get("running").asBoolean());
		assertTrue(json.get("forward").asBoolean());
		assertFalse(json.get("skip").asBoolean());
	}

	@Test
	public void roundTripWithData() {
		ServerCommandReplayStatus cmd = new ServerCommandReplayStatus(9, 3, true, false, true);
		JsonObject json = cmd.toJsonValue();
		ServerCommandReplayStatus restored = new ServerCommandReplayStatus().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(9, restored.getCommandNr());
		assertEquals(3, restored.getSpeed());
		assertTrue(restored.isRunning());
		assertFalse(restored.isForward());
		assertTrue(restored.isSkip());
	}

	@Test
	public void roundTripWithDefault() {
		ServerCommandReplayStatus cmd = new ServerCommandReplayStatus();
		JsonObject json = cmd.toJsonValue();
		ServerCommandReplayStatus restored = new ServerCommandReplayStatus().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(0, restored.getCommandNr());
		assertFalse(restored.isRunning());
	}
}
