package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_block_choice.rs tests.
 */
public class ClientCommandBlockChoiceTest {

	@Test
	public void diceIndexStored() {
		ClientCommandBlockChoice cmd = new ClientCommandBlockChoice(2);
		assertEquals(2, cmd.getDiceIndex());
	}

	@Test
	public void defaultIsZero() {
		ClientCommandBlockChoice cmd = new ClientCommandBlockChoice();
		assertEquals(0, cmd.getDiceIndex());
	}

	@Test
	public void negativeIndexStored() {
		ClientCommandBlockChoice cmd = new ClientCommandBlockChoice(-1);
		assertEquals(-1, cmd.getDiceIndex());
	}

	@Test
	public void getIdIsClientBlockChoice() {
		assertEquals(NetCommandId.CLIENT_BLOCK_CHOICE, new ClientCommandBlockChoice(0).getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndDiceIndex() {
		ClientCommandBlockChoice cmd = new ClientCommandBlockChoice(2);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientBlockChoice", json.get("netCommandId").asString());
		assertEquals(2, json.get("diceIndex").asInt());
	}

	@Test
	public void roundTripWithPopulatedData() {
		ClientCommandBlockChoice cmd = new ClientCommandBlockChoice(3);
		cmd.setEntropy((byte) 11);
		JsonObject json = cmd.toJsonValue();
		ClientCommandBlockChoice restored = new ClientCommandBlockChoice().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 11, restored.getEntropy());
		assertEquals(3, restored.getDiceIndex());
	}

	@Test
	public void roundTripWithDefaultData() {
		ClientCommandBlockChoice cmd = new ClientCommandBlockChoice();
		JsonObject json = cmd.toJsonValue();
		ClientCommandBlockChoice restored = new ClientCommandBlockChoice().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(0, restored.getDiceIndex());
	}
}
