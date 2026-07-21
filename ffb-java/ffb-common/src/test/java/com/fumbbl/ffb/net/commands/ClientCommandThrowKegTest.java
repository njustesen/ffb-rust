package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_throw_keg.rs tests.
 */
public class ClientCommandThrowKegTest {

	@Test
	public void defaultHasNoPlayerId() {
		ClientCommandThrowKeg cmd = new ClientCommandThrowKeg();
		assertNull(cmd.getPlayerId());
	}

	@Test
	public void withPlayerIdStoresValue() {
		ClientCommandThrowKeg cmd = new ClientCommandThrowKeg("p-42");
		assertEquals("p-42", cmd.getPlayerId());
	}

	@Test
	public void getIdIsClientThrowKeg() {
		assertEquals(NetCommandId.CLIENT_THROW_KEG, new ClientCommandThrowKeg().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndPlayerId() {
		ClientCommandThrowKeg cmd = new ClientCommandThrowKeg("p-1");
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientThrowKeg", json.get("netCommandId").asString());
		assertEquals("p-1", json.get("playerId").asString());
	}

	@Test
	public void roundTripWithPlayerIdAndEntropy() {
		ClientCommandThrowKeg cmd = new ClientCommandThrowKeg("p-2");
		cmd.setEntropy((byte) 5);
		JsonObject json = cmd.toJsonValue();
		ClientCommandThrowKeg restored = new ClientCommandThrowKeg().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 5, restored.getEntropy());
		assertEquals("p-2", restored.getPlayerId());
	}

	@Test
	public void roundTripWithNoPlayerId() {
		ClientCommandThrowKeg cmd = new ClientCommandThrowKeg();
		JsonObject json = cmd.toJsonValue();
		ClientCommandThrowKeg restored = new ClientCommandThrowKeg().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getPlayerId());
	}
}
