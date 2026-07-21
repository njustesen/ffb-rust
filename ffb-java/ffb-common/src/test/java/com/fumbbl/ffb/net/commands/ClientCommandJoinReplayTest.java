package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_join_replay.rs tests.
 */
public class ClientCommandJoinReplayTest {

	@Test
	public void defaultGameIdIsZero() {
		ClientCommandJoinReplay cmd = new ClientCommandJoinReplay();
		assertEquals(0L, cmd.getGameId());
	}

	@Test
	public void storesReplayNameAndCoach() {
		ClientCommandJoinReplay cmd = new ClientCommandJoinReplay("replay_001", "CoachA", 99);
		assertEquals("replay_001", cmd.getReplayName());
		assertEquals("CoachA", cmd.getCoach());
		assertEquals(99L, cmd.getGameId());
	}

	@Test
	public void replayNameNoneByDefault() {
		ClientCommandJoinReplay cmd = new ClientCommandJoinReplay();
		assertNull(cmd.getReplayName());
	}

	@Test
	public void getIdIsClientJoinReplay() {
		assertEquals(NetCommandId.CLIENT_JOIN_REPLAY, new ClientCommandJoinReplay().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndName() {
		ClientCommandJoinReplay cmd = new ClientCommandJoinReplay("replay_001", null, 5);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientJoinReplay", json.get("netCommandId").asString());
		assertEquals("replay_001", json.get("name").asString());
	}

	@Test
	public void roundTripWithFieldsAndEntropy() {
		ClientCommandJoinReplay cmd = new ClientCommandJoinReplay("replay_001", "CoachA", 99);
		cmd.setEntropy((byte) 9);
		JsonObject json = cmd.toJsonValue();
		ClientCommandJoinReplay restored = new ClientCommandJoinReplay().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 9, restored.getEntropy());
		assertEquals("replay_001", restored.getReplayName());
		assertEquals("CoachA", restored.getCoach());
		assertEquals(99L, restored.getGameId());
	}

	@Test
	public void roundTripDefault() {
		ClientCommandJoinReplay cmd = new ClientCommandJoinReplay();
		JsonObject json = cmd.toJsonValue();
		ClientCommandJoinReplay restored = new ClientCommandJoinReplay().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getReplayName());
		assertEquals(0L, restored.getGameId());
	}
}
