package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_replay.rs tests.
 */
public class ClientCommandReplayTest {

	@Test
	public void fieldsStored() {
		ClientCommandReplay cmd = new ClientCommandReplay(42, 100, "coach1");
		assertEquals(42L, cmd.getGameId());
		assertEquals(100, cmd.getReplayToCommandNr());
		assertEquals("coach1", cmd.getCoach());
	}

	@Test
	public void defaultIsZeroed() {
		ClientCommandReplay cmd = new ClientCommandReplay();
		assertEquals(0L, cmd.getGameId());
		assertEquals(0, cmd.getReplayToCommandNr());
		assertNull(cmd.getCoach());
	}

	@Test
	public void largeGameIdStored() {
		ClientCommandReplay cmd = new ClientCommandReplay(Long.MAX_VALUE, 0, "coach");
		assertEquals(Long.MAX_VALUE, cmd.getGameId());
	}

	@Test
	public void getIdIsClientReplay() {
		assertEquals(NetCommandId.CLIENT_REPLAY, new ClientCommandReplay().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndFields() {
		ClientCommandReplay cmd = new ClientCommandReplay(7, 3, "coachX");
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientReplay", json.get("netCommandId").asString());
		assertEquals(7L, json.get("gameId").asLong());
		assertEquals(3, json.get("replayToCommandNr").asInt());
		assertEquals("coachX", json.get("coach").asString());
	}

	@Test
	public void roundTripWithData() {
		ClientCommandReplay cmd = new ClientCommandReplay(99, 12, "coachY");
		cmd.setEntropy((byte) 2);
		JsonObject json = cmd.toJsonValue();
		ClientCommandReplay restored = new ClientCommandReplay().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 2, restored.getEntropy());
		assertEquals(99L, restored.getGameId());
		assertEquals(12, restored.getReplayToCommandNr());
		assertEquals("coachY", restored.getCoach());
	}

	@Test
	public void roundTripDefault() {
		ClientCommandReplay cmd = new ClientCommandReplay();
		JsonObject json = cmd.toJsonValue();
		ClientCommandReplay restored = new ClientCommandReplay().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(0L, restored.getGameId());
		assertEquals(0, restored.getReplayToCommandNr());
		assertNull(restored.getCoach());
		assertFalse(restored.hasEntropy());
	}
}
