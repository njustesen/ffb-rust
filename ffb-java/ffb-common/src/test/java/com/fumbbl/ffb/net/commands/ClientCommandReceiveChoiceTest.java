package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_receive_choice.rs tests.
 */
public class ClientCommandReceiveChoiceTest {

	@Test
	public void receiveTrueStored() {
		ClientCommandReceiveChoice cmd = new ClientCommandReceiveChoice(true);
		assertTrue(cmd.isChoiceReceive());
	}

	@Test
	public void kickStored() {
		ClientCommandReceiveChoice cmd = new ClientCommandReceiveChoice(false);
		assertFalse(cmd.isChoiceReceive());
	}

	@Test
	public void defaultIsKick() {
		ClientCommandReceiveChoice cmd = new ClientCommandReceiveChoice();
		assertFalse(cmd.isChoiceReceive());
	}

	@Test
	public void getIdIsClientReceiveChoice() {
		assertEquals(NetCommandId.CLIENT_RECEIVE_CHOICE, new ClientCommandReceiveChoice().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndChoiceReceive() {
		ClientCommandReceiveChoice cmd = new ClientCommandReceiveChoice(true);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientReceiveChoice", json.get("netCommandId").asString());
		assertTrue(json.get("choiceReceive").asBoolean());
	}

	@Test
	public void roundTripWithData() {
		ClientCommandReceiveChoice cmd = new ClientCommandReceiveChoice(true);
		cmd.setEntropy((byte) 7);
		JsonObject json = cmd.toJsonValue();
		ClientCommandReceiveChoice restored = new ClientCommandReceiveChoice().initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 7, restored.getEntropy());
		assertTrue(restored.isChoiceReceive());
	}

	@Test
	public void roundTripDefault() {
		ClientCommandReceiveChoice cmd = new ClientCommandReceiveChoice();
		JsonObject json = cmd.toJsonValue();
		ClientCommandReceiveChoice restored = new ClientCommandReceiveChoice().initFrom(NetCommandTestUtil.gameSource(), json);
		assertFalse(restored.isChoiceReceive());
		assertFalse(restored.hasEntropy());
	}
}
