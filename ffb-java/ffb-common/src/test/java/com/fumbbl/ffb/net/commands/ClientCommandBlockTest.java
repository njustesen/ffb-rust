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
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_block.rs tests.
 */
public class ClientCommandBlockTest {

	@Test
	public void fieldsStoredCorrectly() {
		ClientCommandBlock cmd = new ClientCommandBlock("atk1", "def1", true, false, false, false, false);
		assertEquals("atk1", cmd.getActingPlayerId());
		assertEquals("def1", cmd.getDefenderId());
		assertTrue(cmd.isUsingStab());
		assertFalse(cmd.isUsingChainsaw());
	}

	@Test
	public void chainsawFlag() {
		ClientCommandBlock cmd = new ClientCommandBlock("a", "b", false, true, false, false, false);
		assertTrue(cmd.isUsingChainsaw());
		assertFalse(cmd.isUsingStab());
	}

	@Test
	public void defaultAllFalse() {
		ClientCommandBlock cmd = new ClientCommandBlock();
		assertFalse(cmd.isUsingStab());
		assertFalse(cmd.isUsingChainsaw());
		assertFalse(cmd.isUsingVomit());
		assertFalse(cmd.isUsingBreatheFire());
		assertFalse(cmd.isUsingChomp());
	}

	@Test
	public void getIdIsClientBlock() {
		assertEquals(NetCommandId.CLIENT_BLOCK, new ClientCommandBlock().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndUsingStab() {
		ClientCommandBlock cmd = new ClientCommandBlock("atk1", "def1", true, false, false, false, false);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientBlock", json.get("netCommandId").asString());
		assertTrue(json.get("usingStab").asBoolean());
	}

	@Test
	public void roundTripWithPopulatedData() {
		ClientCommandBlock cmd = new ClientCommandBlock("atk1", "def1", true, true, true, true, true);
		cmd.setEntropy((byte) 5);
		JsonObject json = cmd.toJsonValue();
		ClientCommandBlock restored = new ClientCommandBlock().initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 5, restored.getEntropy());
		assertEquals("atk1", restored.getActingPlayerId());
		assertEquals("def1", restored.getDefenderId());
		assertTrue(restored.isUsingStab());
		assertTrue(restored.isUsingChainsaw());
		assertTrue(restored.isUsingVomit());
		assertTrue(restored.isUsingBreatheFire());
		assertTrue(restored.isUsingChomp());
	}

	@Test
	public void roundTripWithDefaultData() {
		ClientCommandBlock cmd = new ClientCommandBlock();
		JsonObject json = cmd.toJsonValue();
		ClientCommandBlock restored = new ClientCommandBlock().initFrom(NetCommandTestUtil.gameSource(), json);
		assertFalse(restored.hasEntropy());
		assertNull(restored.getActingPlayerId());
		assertNull(restored.getDefenderId());
		assertFalse(restored.isUsingStab());
		assertFalse(restored.isUsingChainsaw());
	}
}
