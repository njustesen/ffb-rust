package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_setup_player.rs tests.
 */
public class ClientCommandSetupPlayerTest {

	@Test
	public void fieldsStoredCorrectly() {
		FieldCoordinate coord = new FieldCoordinate(5, 5);
		ClientCommandSetupPlayer cmd = new ClientCommandSetupPlayer("p1", coord);
		assertEquals("p1", cmd.getPlayerId());
		assertEquals(coord, cmd.getCoordinate());
	}

	@Test
	public void defaultBothNone() {
		ClientCommandSetupPlayer cmd = new ClientCommandSetupPlayer();
		assertNull(cmd.getPlayerId());
		assertNull(cmd.getCoordinate());
	}

	@Test
	public void getIdIsClientSetupPlayer() {
		assertEquals(NetCommandId.CLIENT_SETUP_PLAYER, new ClientCommandSetupPlayer().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndPlayerId() {
		ClientCommandSetupPlayer cmd = new ClientCommandSetupPlayer("p1", new FieldCoordinate(5, 5));
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientSetupPlayer", json.get("netCommandId").asString());
		assertEquals("p1", json.get("playerId").asString());
	}

	@Test
	public void roundTripWithData() {
		ClientCommandSetupPlayer cmd = new ClientCommandSetupPlayer("p2", new FieldCoordinate(3, 7));
		cmd.setEntropy((byte) 9);
		JsonObject json = cmd.toJsonValue();
		ClientCommandSetupPlayer restored = new ClientCommandSetupPlayer().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals("p2", restored.getPlayerId());
		assertEquals(new FieldCoordinate(3, 7), restored.getCoordinate());
		assertEquals((byte) 9, restored.getEntropy());
	}

	@Test
	public void roundTripDefault() {
		ClientCommandSetupPlayer cmd = new ClientCommandSetupPlayer();
		JsonObject json = cmd.toJsonValue();
		ClientCommandSetupPlayer restored = new ClientCommandSetupPlayer().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getPlayerId());
		assertNull(restored.getCoordinate());
		assertFalse(restored.hasEntropy());
	}
}
