package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_pick_up_choice.rs tests.
 * Java exposes the attemptPickUp field via {@code isChoicePickUp()}.
 */
public class ClientCommandPickUpChoiceTest {

	@Test
	public void newTrueStoresTrue() {
		ClientCommandPickUpChoice cmd = new ClientCommandPickUpChoice(true);
		assertTrue(cmd.isChoicePickUp());
	}

	@Test
	public void newFalseStoresFalse() {
		ClientCommandPickUpChoice cmd = new ClientCommandPickUpChoice(false);
		assertFalse(cmd.isChoicePickUp());
	}

	@Test
	public void defaultIsFalse() {
		ClientCommandPickUpChoice cmd = new ClientCommandPickUpChoice();
		assertFalse(cmd.isChoicePickUp());
	}

	@Test
	public void getIdIsClientPickUpChoice() {
		assertEquals(NetCommandId.CLIENT_PICK_UP_CHOICE, new ClientCommandPickUpChoice().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndChoicePickUp() {
		ClientCommandPickUpChoice cmd = new ClientCommandPickUpChoice(true);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientPickUpChoice", json.get("netCommandId").asString());
		assertTrue(json.get("choicePickUp").asBoolean());
	}

	@Test
	public void roundTripWithData() {
		ClientCommandPickUpChoice cmd = new ClientCommandPickUpChoice(true);
		cmd.setEntropy((byte) 5);
		JsonObject json = cmd.toJsonValue();
		ClientCommandPickUpChoice restored = new ClientCommandPickUpChoice().initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 5, restored.getEntropy());
		assertTrue(restored.isChoicePickUp());
	}

	@Test
	public void roundTripDefault() {
		ClientCommandPickUpChoice cmd = new ClientCommandPickUpChoice();
		JsonObject json = cmd.toJsonValue();
		ClientCommandPickUpChoice restored = new ClientCommandPickUpChoice().initFrom(NetCommandTestUtil.gameSource(), json);
		assertFalse(restored.isChoicePickUp());
		assertFalse(restored.hasEntropy());
	}
}
