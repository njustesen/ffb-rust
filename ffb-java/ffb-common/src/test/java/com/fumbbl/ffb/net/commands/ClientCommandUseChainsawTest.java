package com.fumbbl.ffb.net.commands;

import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_use_chainsaw.rs tests.
 */
public class ClientCommandUseChainsawTest {

	@Test
	public void newTrueStoresTrue() {
		ClientCommandUseChainsaw cmd = new ClientCommandUseChainsaw(true);
		assertTrue(cmd.isUsingChainsaw());
	}

	@Test
	public void newFalseStoresFalse() {
		ClientCommandUseChainsaw cmd = new ClientCommandUseChainsaw(false);
		assertFalse(cmd.isUsingChainsaw());
	}

	@Test
	public void defaultIsFalse() {
		ClientCommandUseChainsaw cmd = new ClientCommandUseChainsaw();
		assertFalse(cmd.isUsingChainsaw());
	}

	@Test
	public void getIdIsClientUseChainsaw() {
		assertEquals(NetCommandId.CLIENT_USE_CHAINSAW, new ClientCommandUseChainsaw().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndUsingChainsaw() {
		ClientCommandUseChainsaw cmd = new ClientCommandUseChainsaw(true);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientUseChainsaw", json.get("netCommandId").asString());
		assertTrue(json.get("usingChainsaw").asBoolean());
	}

	@Test
	public void roundTripWithTrueAndEntropy() {
		ClientCommandUseChainsaw cmd = new ClientCommandUseChainsaw(true);
		cmd.setEntropy((byte) 1);
		JsonObject json = cmd.toJsonValue();
		ClientCommandUseChainsaw restored = (ClientCommandUseChainsaw) new ClientCommandUseChainsaw()
			.initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 1, restored.getEntropy());
		assertTrue(restored.isUsingChainsaw());
	}

	@Test
	public void roundTripWithDefaultFalse() {
		ClientCommandUseChainsaw cmd = new ClientCommandUseChainsaw();
		JsonObject json = cmd.toJsonValue();
		ClientCommandUseChainsaw restored = (ClientCommandUseChainsaw) new ClientCommandUseChainsaw()
			.initFrom(NetCommandTestUtil.gameSource(), json);
		assertFalse(restored.isUsingChainsaw());
	}
}
