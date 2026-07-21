package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_replay_status.rs tests.
 */
public class ClientCommandReplayStatusTest {

	@Test
	public void fieldsStored() {
		ClientCommandReplayStatus cmd = new ClientCommandReplayStatus(55, 2, true, true, false);
		assertEquals(55, cmd.getCommandNr());
		assertEquals(2, cmd.getSpeed());
		assertTrue(cmd.isRunning());
		assertTrue(cmd.isForward());
		assertFalse(cmd.isSkip());
	}

	@Test
	public void defaultIsZeroed() {
		ClientCommandReplayStatus cmd = new ClientCommandReplayStatus();
		assertEquals(0, cmd.getCommandNr());
		assertFalse(cmd.isRunning());
		assertFalse(cmd.isSkip());
	}

	@Test
	public void skipCanBeSet() {
		ClientCommandReplayStatus cmd = new ClientCommandReplayStatus(0, 1, false, false, true);
		assertTrue(cmd.isSkip());
		assertFalse(cmd.isRunning());
	}

	@Test
	public void getIdIsClientReplayStatus() {
		assertEquals(NetCommandId.CLIENT_REPLAY_STATUS, new ClientCommandReplayStatus().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndFields() {
		ClientCommandReplayStatus cmd = new ClientCommandReplayStatus(10, 3, true, false, true);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientReplayStatus", json.get("netCommandId").asString());
		assertEquals(10, json.get("commandNr").asInt());
		assertEquals(3, json.get("speed").asInt());
		assertTrue(json.get("running").asBoolean());
		assertFalse(json.get("forward").asBoolean());
		assertTrue(json.get("skip").asBoolean());
	}

	@Test
	public void roundTripWithData() {
		ClientCommandReplayStatus cmd = new ClientCommandReplayStatus(20, 5, true, true, false);
		cmd.setEntropy((byte) 1);
		JsonObject json = cmd.toJsonValue();
		ClientCommandReplayStatus restored = new ClientCommandReplayStatus().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 1, restored.getEntropy());
		assertEquals(20, restored.getCommandNr());
		assertEquals(5, restored.getSpeed());
		assertTrue(restored.isRunning());
		assertTrue(restored.isForward());
		assertFalse(restored.isSkip());
	}

	@Test
	public void roundTripDefault() {
		ClientCommandReplayStatus cmd = new ClientCommandReplayStatus();
		JsonObject json = cmd.toJsonValue();
		ClientCommandReplayStatus restored = new ClientCommandReplayStatus().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(0, restored.getCommandNr());
		assertEquals(0, restored.getSpeed());
		assertFalse(restored.isRunning());
		assertFalse(restored.isForward());
		assertFalse(restored.isSkip());
		assertFalse(restored.hasEntropy());
	}
}
