package com.fumbbl.ffb.net.commands;

import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_use_pro_re_roll_for_block.rs tests.
 */
public class ClientCommandUseProReRollForBlockTest {

	@Test
	public void indexStored() {
		assertEquals(2, new ClientCommandUseProReRollForBlock(2).getProIndex());
	}

	@Test
	public void defaultZero() {
		assertEquals(0, new ClientCommandUseProReRollForBlock().getProIndex());
	}

	@Test
	public void getIdIsClientUseProReRollForBlock() {
		assertEquals(NetCommandId.CLIENT_USE_PRO_RE_ROLL_FOR_BLOCK,
			new ClientCommandUseProReRollForBlock(0).getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndProIndex() {
		ClientCommandUseProReRollForBlock cmd = new ClientCommandUseProReRollForBlock(2);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientUseProReRollForBlock", json.get("netCommandId").asString());
		assertEquals(2, json.get("proIndex").asInt());
	}

	@Test
	public void roundTripWithIndexAndEntropy() {
		ClientCommandUseProReRollForBlock cmd = new ClientCommandUseProReRollForBlock(3);
		cmd.setEntropy((byte) 6);
		JsonObject json = cmd.toJsonValue();
		ClientCommandUseProReRollForBlock restored = new ClientCommandUseProReRollForBlock()
			.initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 6, restored.getEntropy());
		assertEquals(3, restored.getProIndex());
	}

	@Test
	public void roundTripWithDefaultIndex() {
		ClientCommandUseProReRollForBlock cmd = new ClientCommandUseProReRollForBlock();
		JsonObject json = cmd.toJsonValue();
		ClientCommandUseProReRollForBlock restored = new ClientCommandUseProReRollForBlock()
			.initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(0, restored.getProIndex());
	}
}
