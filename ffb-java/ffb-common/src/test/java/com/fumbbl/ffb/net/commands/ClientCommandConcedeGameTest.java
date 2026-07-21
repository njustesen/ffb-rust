package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.ConcedeGameStatus;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_concede_game.rs tests.
 */
public class ClientCommandConcedeGameTest {

	@Test
	public void defaultStatusNone() {
		ClientCommandConcedeGame cmd = new ClientCommandConcedeGame();
		assertNull(cmd.getConcedeGameStatus());
	}

	@Test
	public void storesConcedeStatus() {
		ClientCommandConcedeGame cmd = new ClientCommandConcedeGame(ConcedeGameStatus.REQUESTED);
		assertNotNull(cmd.getConcedeGameStatus());
	}

	@Test
	public void getIdIsClientConcedeGame() {
		assertEquals(NetCommandId.CLIENT_CONCEDE_GAME, new ClientCommandConcedeGame().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndConcedeGameStatus() {
		ClientCommandConcedeGame cmd = new ClientCommandConcedeGame(ConcedeGameStatus.CONFIRMED);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientConcedeGame", json.get("netCommandId").asString());
		assertEquals("confirmed", json.get("concedeGameStatus").asString());
	}

	@Test
	public void roundTripWithStatusAndEntropy() {
		ClientCommandConcedeGame cmd = new ClientCommandConcedeGame(ConcedeGameStatus.DENIED);
		cmd.setEntropy((byte) 2);
		JsonObject json = cmd.toJsonValue();
		ClientCommandConcedeGame restored = new ClientCommandConcedeGame().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 2, restored.getEntropy());
		assertEquals(ConcedeGameStatus.DENIED, restored.getConcedeGameStatus());
	}

	@Test
	public void roundTripWithNoStatus() {
		ClientCommandConcedeGame cmd = new ClientCommandConcedeGame();
		JsonObject json = cmd.toJsonValue();
		ClientCommandConcedeGame restored = new ClientCommandConcedeGame().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getConcedeGameStatus());
	}
}
