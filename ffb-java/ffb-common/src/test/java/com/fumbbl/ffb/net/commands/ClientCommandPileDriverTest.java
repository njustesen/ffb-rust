package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_pile_driver.rs tests.
 */
public class ClientCommandPileDriverTest {

	@Test
	public void defaultHasNoPlayerId() {
		ClientCommandPileDriver cmd = new ClientCommandPileDriver();
		assertNull(cmd.getPlayerId());
	}

	@Test
	public void withPlayerIdStoresValue() {
		ClientCommandPileDriver cmd = new ClientCommandPileDriver("p-7");
		assertEquals("p-7", cmd.getPlayerId());
	}

	@Test
	public void getIdIsClientPileDriver() {
		assertEquals(NetCommandId.CLIENT_PILE_DRIVER, new ClientCommandPileDriver().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndPlayerId() {
		ClientCommandPileDriver cmd = new ClientCommandPileDriver("p-9");
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientPileDriver", json.get("netCommandId").asString());
		assertEquals("p-9", json.get("playerId").asString());
	}

	@Test
	public void roundTripWithData() {
		ClientCommandPileDriver cmd = new ClientCommandPileDriver("p-3");
		cmd.setEntropy((byte) 2);
		JsonObject json = cmd.toJsonValue();
		ClientCommandPileDriver restored =
			(ClientCommandPileDriver) new ClientCommandPileDriver().initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 2, restored.getEntropy());
		assertEquals("p-3", restored.getPlayerId());
	}

	@Test
	public void roundTripDefault() {
		ClientCommandPileDriver cmd = new ClientCommandPileDriver();
		JsonObject json = cmd.toJsonValue();
		ClientCommandPileDriver restored =
			(ClientCommandPileDriver) new ClientCommandPileDriver().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getPlayerId());
		assertFalse(restored.hasEntropy());
	}
}
