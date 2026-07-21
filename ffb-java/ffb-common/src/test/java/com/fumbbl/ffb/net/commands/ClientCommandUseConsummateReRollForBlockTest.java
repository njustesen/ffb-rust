package com.fumbbl.ffb.net.commands;

import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_use_consummate_re_roll_for_block.rs tests.
 */
public class ClientCommandUseConsummateReRollForBlockTest {

	@Test
	public void newStoresIndex() {
		ClientCommandUseConsummateReRollForBlock cmd = new ClientCommandUseConsummateReRollForBlock(3);
		assertEquals(3, cmd.getProIndex());
	}

	@Test
	public void defaultIsZero() {
		ClientCommandUseConsummateReRollForBlock cmd = new ClientCommandUseConsummateReRollForBlock();
		assertEquals(0, cmd.getProIndex());
	}

	@Test
	public void getIdIsClientUseConsummateReRollForBlock() {
		assertEquals(NetCommandId.CLIENT_USE_CONSUMMATE_RE_ROLL_FOR_BLOCK,
			new ClientCommandUseConsummateReRollForBlock().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndProIndex() {
		ClientCommandUseConsummateReRollForBlock cmd = new ClientCommandUseConsummateReRollForBlock(4);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientUseConsummateReRollForBlock", json.get("netCommandId").asString());
		assertEquals(4, json.get("proIndex").asInt());
	}

	@Test
	public void roundTripWithIndexAndEntropy() {
		ClientCommandUseConsummateReRollForBlock cmd = new ClientCommandUseConsummateReRollForBlock(7);
		cmd.setEntropy((byte) 3);
		JsonObject json = cmd.toJsonValue();
		ClientCommandUseConsummateReRollForBlock restored = new ClientCommandUseConsummateReRollForBlock()
			.initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 3, restored.getEntropy());
		assertEquals(7, restored.getProIndex());
	}

	@Test
	public void roundTripWithDefaultZero() {
		ClientCommandUseConsummateReRollForBlock cmd = new ClientCommandUseConsummateReRollForBlock();
		JsonObject json = cmd.toJsonValue();
		ClientCommandUseConsummateReRollForBlock restored = new ClientCommandUseConsummateReRollForBlock()
			.initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(0, restored.getProIndex());
	}
}
