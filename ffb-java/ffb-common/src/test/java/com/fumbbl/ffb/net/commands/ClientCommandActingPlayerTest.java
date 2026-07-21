package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_acting_player.rs tests.
 */
public class ClientCommandActingPlayerTest {

	@Test
	public void fieldsStored() {
		ClientCommandActingPlayer cmd = new ClientCommandActingPlayer("p1", PlayerAction.MOVE, false);
		assertEquals("p1", cmd.getPlayerId());
		assertEquals(PlayerAction.MOVE, cmd.getPlayerAction());
		assertFalse(cmd.isJumping());
	}

	@Test
	public void jumpingFlag() {
		ClientCommandActingPlayer cmd = new ClientCommandActingPlayer("p2", PlayerAction.BLOCK, true);
		assertTrue(cmd.isJumping());
	}

	@Test
	public void defaultIsEmpty() {
		ClientCommandActingPlayer cmd = new ClientCommandActingPlayer();
		assertNull(cmd.getPlayerId());
		assertNull(cmd.getPlayerAction());
		assertFalse(cmd.isJumping());
	}

	@Test
	public void getIdIsClientActingPlayer() {
		assertEquals(NetCommandId.CLIENT_ACTING_PLAYER, new ClientCommandActingPlayer().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndPlayerAction() {
		ClientCommandActingPlayer cmd = new ClientCommandActingPlayer("p1", PlayerAction.BLOCK, true);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientActingPlayer", json.get("netCommandId").asString());
		assertEquals("block", json.get("playerAction").asString());
		assertTrue(json.get("leaping").asBoolean());
	}

	@Test
	public void roundTripWithPopulatedFields() {
		ClientCommandActingPlayer cmd = new ClientCommandActingPlayer("p1", PlayerAction.BLITZ, true);
		cmd.setEntropy((byte) 5);
		JsonObject json = cmd.toJsonValue();
		ClientCommandActingPlayer restored = new ClientCommandActingPlayer().initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 5, restored.getEntropy());
		assertEquals("p1", restored.getPlayerId());
		assertEquals(PlayerAction.BLITZ, restored.getPlayerAction());
		assertTrue(restored.isJumping());
	}

	@Test
	public void roundTripWithDefaultData() {
		ClientCommandActingPlayer cmd = new ClientCommandActingPlayer();
		JsonObject json = cmd.toJsonValue();
		ClientCommandActingPlayer restored = new ClientCommandActingPlayer().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getPlayerId());
		assertNull(restored.getPlayerAction());
		assertFalse(restored.isJumping());
	}
}
