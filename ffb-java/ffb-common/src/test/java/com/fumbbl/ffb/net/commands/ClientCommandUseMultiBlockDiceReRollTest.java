package com.fumbbl.ffb.net.commands;

import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonArray;
import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_use_multi_block_dice_re_roll.rs tests.
 */
public class ClientCommandUseMultiBlockDiceReRollTest {

	@Test
	public void indexesStored() {
		ClientCommandUseMultiBlockDiceReRoll cmd = new ClientCommandUseMultiBlockDiceReRoll(new int[] { 0, 2 });
		assertArrayEquals(new int[] { 0, 2 }, cmd.getDiceIndexes());
	}

	@Test
	public void defaultEmpty() {
		assertNull(new ClientCommandUseMultiBlockDiceReRoll().getDiceIndexes());
	}

	@Test
	public void getIdIsClientUseMultiBlockDiceReRoll() {
		assertEquals(NetCommandId.CLIENT_USE_MULTI_BLOCK_DICE_RE_ROLL,
			new ClientCommandUseMultiBlockDiceReRoll().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndIndexes() {
		ClientCommandUseMultiBlockDiceReRoll cmd = new ClientCommandUseMultiBlockDiceReRoll(new int[] { 1, 3 });
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientUseMultiBlockDiceReRoll", json.get("netCommandId").asString());
		JsonArray array = json.get("blockDiceIndexes").asArray();
		assertEquals(2, array.size());
		assertEquals(1, array.get(0).asInt());
		assertEquals(3, array.get(1).asInt());
	}

	@Test
	public void roundTripWithIndexesAndEntropy() {
		ClientCommandUseMultiBlockDiceReRoll cmd = new ClientCommandUseMultiBlockDiceReRoll(new int[] { 0, 1, 2 });
		cmd.setEntropy((byte) 8);
		JsonObject json = cmd.toJsonValue();
		ClientCommandUseMultiBlockDiceReRoll restored = new ClientCommandUseMultiBlockDiceReRoll()
			.initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 8, restored.getEntropy());
		assertArrayEquals(new int[] { 0, 1, 2 }, restored.getDiceIndexes());
	}

	@Test
	public void roundTripWithNoIndexes() {
		ClientCommandUseMultiBlockDiceReRoll cmd = new ClientCommandUseMultiBlockDiceReRoll();
		JsonObject json = cmd.toJsonValue();
		ClientCommandUseMultiBlockDiceReRoll restored = new ClientCommandUseMultiBlockDiceReRoll()
			.initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getDiceIndexes());
	}
}
