package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_gaze.rs tests.
 */
public class ClientCommandGazeTest {

	@Test
	public void fieldsStoredCorrectly() {
		ClientCommandGaze cmd = new ClientCommandGaze("gazer", "victim");
		assertEquals("gazer", cmd.getActingPlayerId());
		assertEquals("victim", cmd.getVictimId());
	}

	@Test
	public void defaultBothNone() {
		ClientCommandGaze cmd = new ClientCommandGaze();
		assertNull(cmd.getActingPlayerId());
		assertNull(cmd.getVictimId());
	}

	@Test
	public void newConstructorCreatesDefault() {
		ClientCommandGaze cmd = new ClientCommandGaze();
		assertNotNull(cmd);
	}

	@Test
	public void victimIdStored() {
		ClientCommandGaze cmd = new ClientCommandGaze("g", "v");
		assertEquals("v", cmd.getVictimId());
	}

	@Test
	public void getIdIsClientGaze() {
		assertEquals(NetCommandId.CLIENT_GAZE, new ClientCommandGaze().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndVictimId() {
		ClientCommandGaze cmd = new ClientCommandGaze("g", "v");
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientGaze", json.get("netCommandId").asString());
		assertEquals("v", json.get("victimId").asString());
	}

	@Test
	public void roundTripWithPlayersAndEntropy() {
		ClientCommandGaze cmd = new ClientCommandGaze("gazer", "victim");
		cmd.setEntropy((byte) 4);
		JsonObject json = cmd.toJsonValue();
		ClientCommandGaze restored = new ClientCommandGaze().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 4, restored.getEntropy());
		assertEquals("gazer", restored.getActingPlayerId());
		assertEquals("victim", restored.getVictimId());
	}

	@Test
	public void roundTripDefault() {
		ClientCommandGaze cmd = new ClientCommandGaze();
		JsonObject json = cmd.toJsonValue();
		ClientCommandGaze restored = new ClientCommandGaze().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getActingPlayerId());
		assertNull(restored.getVictimId());
	}
}
