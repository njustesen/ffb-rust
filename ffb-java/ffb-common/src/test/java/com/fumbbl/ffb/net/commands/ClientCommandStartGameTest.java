package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.FactoryType.FactoryContext;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_start_game.rs tests.
 */
public class ClientCommandStartGameTest {

	@Test
	public void getIdIsClientStartGame() {
		assertEquals(NetCommandId.CLIENT_START_GAME, new ClientCommandStartGame().getId());
	}

	@Test
	public void getContextIsApplication() {
		assertEquals(FactoryContext.APPLICATION, new ClientCommandStartGame().getContext());
	}

	@Test
	public void toJsonValueHasNetCommandId() {
		ClientCommandStartGame cmd = new ClientCommandStartGame();
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientStartGame", json.get("netCommandId").asString());
	}

	@Test
	public void roundTripWithEntropy() {
		ClientCommandStartGame cmd = new ClientCommandStartGame();
		cmd.setEntropy((byte) 5);
		JsonObject json = cmd.toJsonValue();
		ClientCommandStartGame restored = new ClientCommandStartGame().initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 5, restored.getEntropy());
	}

	@Test
	public void roundTripWithNoEntropy() {
		ClientCommandStartGame cmd = new ClientCommandStartGame();
		JsonObject json = cmd.toJsonValue();
		ClientCommandStartGame restored = new ClientCommandStartGame().initFrom(NetCommandTestUtil.gameSource(), json);
		assertFalse(restored.hasEntropy());
	}
}
