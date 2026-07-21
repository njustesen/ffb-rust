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
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_foul.rs tests.
 */
public class ClientCommandFoulTest {

	@Test
	public void fieldsStoredCorrectly() {
		ClientCommandFoul cmd = new ClientCommandFoul("atk", "def", true);
		assertEquals("atk", cmd.getActingPlayerId());
		assertEquals("def", cmd.getDefenderId());
		assertTrue(cmd.isUsingChainsaw());
	}

	@Test
	public void defaultAllNoneAndFalse() {
		ClientCommandFoul cmd = new ClientCommandFoul();
		assertNull(cmd.getActingPlayerId());
		assertNull(cmd.getDefenderId());
		assertFalse(cmd.isUsingChainsaw());
	}

	@Test
	public void noChainsawFlag() {
		ClientCommandFoul cmd = new ClientCommandFoul("a", "b", false);
		assertFalse(cmd.isUsingChainsaw());
	}

	@Test
	public void getIdIsClientFoul() {
		assertEquals(NetCommandId.CLIENT_FOUL, new ClientCommandFoul().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndUsingChainsaw() {
		ClientCommandFoul cmd = new ClientCommandFoul("atk", "def", true);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientFoul", json.get("netCommandId").asString());
		assertTrue(json.get("usingChainsaw").asBoolean());
		assertEquals("atk", json.get("actingPlayerId").asString());
	}

	@Test
	public void roundTripWithPlayersAndEntropy() {
		ClientCommandFoul cmd = new ClientCommandFoul("atk", "def", true);
		cmd.setEntropy((byte) 6);
		JsonObject json = cmd.toJsonValue();
		ClientCommandFoul restored = new ClientCommandFoul().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 6, restored.getEntropy());
		assertEquals("atk", restored.getActingPlayerId());
		assertEquals("def", restored.getDefenderId());
		assertTrue(restored.isUsingChainsaw());
	}

	@Test
	public void roundTripDefault() {
		ClientCommandFoul cmd = new ClientCommandFoul();
		JsonObject json = cmd.toJsonValue();
		ClientCommandFoul restored = new ClientCommandFoul().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getActingPlayerId());
		assertNull(restored.getDefenderId());
		assertFalse(restored.isUsingChainsaw());
	}
}
