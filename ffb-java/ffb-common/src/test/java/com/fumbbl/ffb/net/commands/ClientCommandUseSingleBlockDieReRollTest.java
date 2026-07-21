package com.fumbbl.ffb.net.commands;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.ReRollSource;
import com.fumbbl.ffb.factory.ReRollSourceFactory;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_use_single_block_die_re_roll.rs tests.
 */
public class ClientCommandUseSingleBlockDieReRollTest {

	private static ReRollSource source(String name) {
		ReRollSourceFactory factory = NetCommandTestUtil.gameSource().getFactory(FactoryType.Factory.RE_ROLL_SOURCE);
		return factory.forName(name);
	}

	@Test
	public void dieIndexStored() {
		assertEquals(1, new ClientCommandUseSingleBlockDieReRoll(1).getDieIndex());
	}

	@Test
	public void defaultNoSource() {
		assertNull(new ClientCommandUseSingleBlockDieReRoll(0).getReRollSource());
	}

	@Test
	public void getIdIsClientUseSingleBlockDieReRoll() {
		assertEquals(NetCommandId.CLIENT_USE_SINGLE_BLOCK_DIE_RE_ROLL,
			new ClientCommandUseSingleBlockDieReRoll(0).getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndDieIndex() {
		ClientCommandUseSingleBlockDieReRoll cmd = new ClientCommandUseSingleBlockDieReRoll(1);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientUseSingleBlockDieReRoll", json.get("netCommandId").asString());
		assertEquals(1, json.get("blockDieIndex").asInt());
	}

	@Test
	public void roundTripWithSourceAndEntropy() {
		ReRollSource trr = source("Team ReRoll");
		assertNotNull(trr);
		ClientCommandUseSingleBlockDieReRoll cmd = new ClientCommandUseSingleBlockDieReRoll(2, trr);
		cmd.setEntropy((byte) 9);
		JsonObject json = cmd.toJsonValue();
		ClientCommandUseSingleBlockDieReRoll restored = new ClientCommandUseSingleBlockDieReRoll()
			.initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 9, restored.getEntropy());
		assertEquals(2, restored.getDieIndex());
		assertNotNull(restored.getReRollSource());
		assertEquals(trr.getName(), restored.getReRollSource().getName());
	}

	@Test
	public void roundTripWithNoSource() {
		ClientCommandUseSingleBlockDieReRoll cmd = new ClientCommandUseSingleBlockDieReRoll(0);
		JsonObject json = cmd.toJsonValue();
		ClientCommandUseSingleBlockDieReRoll restored = new ClientCommandUseSingleBlockDieReRoll()
			.initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getReRollSource());
	}
}
