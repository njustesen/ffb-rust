package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_followup_choice.rs tests.
 */
public class ClientCommandFollowupChoiceTest {

	@Test
	public void followupTrueStored() {
		ClientCommandFollowupChoice cmd = new ClientCommandFollowupChoice(true);
		assertTrue(cmd.isChoiceFollowup());
	}

	@Test
	public void followupFalseStored() {
		ClientCommandFollowupChoice cmd = new ClientCommandFollowupChoice(false);
		assertFalse(cmd.isChoiceFollowup());
	}

	@Test
	public void defaultNoFollowup() {
		ClientCommandFollowupChoice cmd = new ClientCommandFollowupChoice();
		assertFalse(cmd.isChoiceFollowup());
	}

	@Test
	public void getIdIsClientFollowupChoice() {
		assertEquals(NetCommandId.CLIENT_FOLLOWUP_CHOICE, new ClientCommandFollowupChoice(true).getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndChoiceFollowup() {
		ClientCommandFollowupChoice cmd = new ClientCommandFollowupChoice(true);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientFollowupChoice", json.get("netCommandId").asString());
		assertTrue(json.get("choiceFollowup").asBoolean());
	}

	@Test
	public void roundTripWithEntropy() {
		ClientCommandFollowupChoice cmd = new ClientCommandFollowupChoice(true);
		cmd.setEntropy((byte) 2);
		JsonObject json = cmd.toJsonValue();
		ClientCommandFollowupChoice restored = new ClientCommandFollowupChoice().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 2, restored.getEntropy());
		assertTrue(restored.isChoiceFollowup());
	}

	@Test
	public void roundTripDefault() {
		ClientCommandFollowupChoice cmd = new ClientCommandFollowupChoice();
		JsonObject json = cmd.toJsonValue();
		ClientCommandFollowupChoice restored = new ClientCommandFollowupChoice().initFrom(NetCommandTestUtil.gameSource(), json);
		assertFalse(restored.isChoiceFollowup());
		assertFalse(restored.hasEntropy());
	}
}
