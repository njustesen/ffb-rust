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
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_use_re_roll_for_target.rs tests.
 * Extends {@link ClientCommandUseReRoll}; adds {@code targetId} serialised under JSON key {@code playerId}.
 */
public class ClientCommandUseReRollForTargetTest {

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
		ClientCommandUseReRollForTarget cmd = new ClientCommandUseReRollForTarget(null, null, "p2");
		assertEquals("p2", cmd.getTargetId());
	}

	@Test
	public void defaultNone() {
		assertNull(new ClientCommandUseReRollForTarget().getTargetId());
	}

	@Test
	public void getIdIsClientUseReRollForTarget() {
		assertEquals(NetCommandId.CLIENT_USE_RE_ROLL_FOR_TARGET, new ClientCommandUseReRollForTarget().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndPlayerId() {
		ClientCommandUseReRollForTarget cmd = new ClientCommandUseReRollForTarget(null, null, "p2");
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientUseReRollForTarget", json.get("netCommandId").asString());
		assertEquals("p2", json.get("playerId").asString());
	}

	@Test
	public void roundTripWithAllFieldsAndEntropy() {
		ReRolledAction block = action("Block");
		ReRollSource trr = source("Team ReRoll");
		assertNotNull(block);
		assertNotNull(trr);
		ClientCommandUseReRollForTarget cmd = new ClientCommandUseReRollForTarget(block, trr, "p3");
		cmd.setEntropy((byte) 2);
		JsonObject json = cmd.toJsonValue();
		ClientCommandUseReRollForTarget restored = new ClientCommandUseReRollForTarget()
			.initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 2, restored.getEntropy());
		assertEquals("p3", restored.getTargetId());
		assertEquals(block.getName(), restored.getReRolledAction().getName());
		assertEquals(trr.getName(), restored.getReRollSource().getName());
	}

	@Test
	public void roundTripWithNoFields() {
		ClientCommandUseReRollForTarget cmd = new ClientCommandUseReRollForTarget();
		JsonObject json = cmd.toJsonValue();
		ClientCommandUseReRollForTarget restored = new ClientCommandUseReRollForTarget()
			.initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getTargetId());
		assertNull(restored.getReRolledAction());
		assertNull(restored.getReRollSource());
	}
}
