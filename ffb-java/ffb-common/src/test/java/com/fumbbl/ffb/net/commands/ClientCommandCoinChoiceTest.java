package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_coin_choice.rs tests.
 */
public class ClientCommandCoinChoiceTest {

	@Test
	public void choiceHeadsStored() {
		ClientCommandCoinChoice cmd = new ClientCommandCoinChoice(true);
		assertTrue(cmd.isChoiceHeads());
	}

	@Test
	public void choiceTailsStored() {
		ClientCommandCoinChoice cmd = new ClientCommandCoinChoice(false);
		assertFalse(cmd.isChoiceHeads());
	}

	@Test
	public void defaultIsTails() {
		ClientCommandCoinChoice cmd = new ClientCommandCoinChoice();
		assertFalse(cmd.isChoiceHeads());
	}

	@Test
	public void getIdIsClientCoinChoice() {
		assertEquals(NetCommandId.CLIENT_COIN_CHOICE, new ClientCommandCoinChoice(true).getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndChoiceHeads() {
		ClientCommandCoinChoice cmd = new ClientCommandCoinChoice(true);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientCoinChoice", json.get("netCommandId").asString());
		assertTrue(json.get("choiceHeads").asBoolean());
	}

	@Test
	public void roundTripWithEntropy() {
		ClientCommandCoinChoice cmd = new ClientCommandCoinChoice(true);
		cmd.setEntropy((byte) 3);
		JsonObject json = cmd.toJsonValue();
		ClientCommandCoinChoice restored = new ClientCommandCoinChoice().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 3, restored.getEntropy());
		assertTrue(restored.isChoiceHeads());
	}

	@Test
	public void roundTripWithDefault() {
		ClientCommandCoinChoice cmd = new ClientCommandCoinChoice();
		JsonObject json = cmd.toJsonValue();
		ClientCommandCoinChoice restored = new ClientCommandCoinChoice().initFrom(NetCommandTestUtil.gameSource(), json);
		assertFalse(restored.hasEntropy());
		assertFalse(restored.isChoiceHeads());
	}
}
