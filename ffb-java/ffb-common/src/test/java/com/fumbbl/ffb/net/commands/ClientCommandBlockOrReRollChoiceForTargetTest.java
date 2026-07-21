package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.ReRollSources;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_block_or_re_roll_choice_for_target.rs tests.
 *
 * Note: Java's anyDiceIndexes field is a plain int[] defaulting to null (Rust uses an empty Vec),
 * so the default round-trip asserts null rather than an empty collection.
 */
public class ClientCommandBlockOrReRollChoiceForTargetTest {

	@Test
	public void defaultSelectedIndexIsMinusOne() {
		ClientCommandBlockOrReRollChoiceForTarget cmd = new ClientCommandBlockOrReRollChoiceForTarget();
		assertEquals(-1, cmd.getSelectedIndex());
	}

	@Test
	public void storesTargetIdAndAnyDiceIndexes() {
		ClientCommandBlockOrReRollChoiceForTarget cmd =
			new ClientCommandBlockOrReRollChoiceForTarget("target_1", 2, 1, null, new int[] { 0, 2 });
		assertEquals("target_1", cmd.getTargetId());
		assertEquals(2, cmd.getSelectedIndex());
		assertArrayEquals(new int[] { 0, 2 }, cmd.getAnyDiceIndexes());
	}

	@Test
	public void getIdIsClientBlockOrReRollChoiceForTarget() {
		assertEquals(NetCommandId.CLIENT_BLOCK_OR_RE_ROLL_CHOICE_FOR_TARGET,
			new ClientCommandBlockOrReRollChoiceForTarget().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndPlayerId() {
		ClientCommandBlockOrReRollChoiceForTarget cmd =
			new ClientCommandBlockOrReRollChoiceForTarget("target_1", -1, 0, null);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientBlockOrReRollChoiceForTarget", json.get("netCommandId").asString());
		assertEquals("target_1", json.get("playerId").asString());
	}

	@Test
	public void roundTripWithPopulatedData() {
		ClientCommandBlockOrReRollChoiceForTarget cmd =
			new ClientCommandBlockOrReRollChoiceForTarget("target_1", 2, 1, ReRollSources.PRO, new int[] { 0, 2 });
		cmd.setEntropy((byte) 4);
		JsonObject json = cmd.toJsonValue();
		ClientCommandBlockOrReRollChoiceForTarget restored =
			new ClientCommandBlockOrReRollChoiceForTarget().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 4, restored.getEntropy());
		assertEquals("target_1", restored.getTargetId());
		assertEquals(2, restored.getSelectedIndex());
		assertEquals(1, restored.getProIndex());
		assertNotNull(restored.getReRollSource());
		assertArrayEquals(new int[] { 0, 2 }, restored.getAnyDiceIndexes());
	}

	@Test
	public void roundTripWithDefaultData() {
		ClientCommandBlockOrReRollChoiceForTarget cmd = new ClientCommandBlockOrReRollChoiceForTarget();
		JsonObject json = cmd.toJsonValue();
		ClientCommandBlockOrReRollChoiceForTarget restored =
			new ClientCommandBlockOrReRollChoiceForTarget().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getTargetId());
		assertEquals(-1, restored.getSelectedIndex());
		assertEquals(0, restored.getProIndex());
		assertNull(restored.getReRollSource());
		assertNull(restored.getAnyDiceIndexes());
	}
}
