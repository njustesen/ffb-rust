package com.fumbbl.ffb.net.commands;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.ReRollSource;
import com.fumbbl.ffb.ReRolledAction;
import com.fumbbl.ffb.factory.ReRollSourceFactory;
import com.fumbbl.ffb.factory.ReRolledActionFactory;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_use_re_roll.rs tests.
 * The Rust port stores action/source as name strings; Java stores typed {@link ReRolledAction}
 * and {@link ReRollSource} objects obtained from the game-context factories.
 */
public class ClientCommandUseReRollTest {

	private static ReRolledAction action(String name) {
		ReRolledActionFactory factory = NetCommandTestUtil.gameSource().getFactory(FactoryType.Factory.RE_ROLLED_ACTION);
		return factory.forName(name);
	}

	private static ReRollSource source(String name) {
		ReRollSourceFactory factory = NetCommandTestUtil.gameSource().getFactory(FactoryType.Factory.RE_ROLL_SOURCE);
		return factory.forName(name);
	}

	@Test
	public void fieldsStored() {
		ReRolledAction dodge = action("Dodge");
		ReRollSource pro = source("Pro");
		assertNotNull(dodge);
		assertNotNull(pro);
		ClientCommandUseReRoll cmd = new ClientCommandUseReRoll(dodge, pro);
		assertEquals(dodge.getName(), cmd.getReRolledAction().getName());
		assertEquals(pro.getName(), cmd.getReRollSource().getName());
	}

	@Test
	public void defaultIsEmpty() {
		ClientCommandUseReRoll cmd = new ClientCommandUseReRoll();
		assertNull(cmd.getReRolledAction());
		assertNull(cmd.getReRollSource());
	}

	@Test
	public void getIdIsClientUseReRoll() {
		assertEquals(NetCommandId.CLIENT_USE_RE_ROLL, new ClientCommandUseReRoll().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndReRolledAction() {
		ReRolledAction dodge = action("Dodge");
		ReRollSource pro = source("Pro");
		assertNotNull(dodge);
		assertNotNull(pro);
		ClientCommandUseReRoll cmd = new ClientCommandUseReRoll(dodge, pro);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientUseReRoll", json.get("netCommandId").asString());
		assertEquals(dodge.getName(), json.get("reRolledAction").asString());
		assertEquals(pro.getName(), json.get("reRollSource").asString());
	}

	@Test
	public void roundTripWithFieldsAndEntropy() {
		ReRolledAction block = action("Block");
		ReRollSource pro = source("Pro");
		assertNotNull(block);
		assertNotNull(pro);
		ClientCommandUseReRoll cmd = new ClientCommandUseReRoll(block, pro);
		cmd.setEntropy((byte) 1);
		JsonObject json = cmd.toJsonValue();
		ClientCommandUseReRoll restored = new ClientCommandUseReRoll()
			.initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 1, restored.getEntropy());
		assertEquals(block.getName(), restored.getReRolledAction().getName());
		assertEquals(pro.getName(), restored.getReRollSource().getName());
	}

	@Test
	public void roundTripWithNoFields() {
		ClientCommandUseReRoll cmd = new ClientCommandUseReRoll();
		JsonObject json = cmd.toJsonValue();
		ClientCommandUseReRoll restored = new ClientCommandUseReRoll()
			.initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getReRolledAction());
		assertNull(restored.getReRollSource());
	}
}
