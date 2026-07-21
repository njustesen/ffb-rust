package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_bloodlust_action.rs tests.
 */
public class ClientCommandBloodlustActionTest {

	@Test
	public void changeTrueStored() {
		ClientCommandBloodlustAction cmd = new ClientCommandBloodlustAction(true);
		assertTrue(cmd.isChange());
	}

	@Test
	public void defaultIsFalse() {
		ClientCommandBloodlustAction cmd = new ClientCommandBloodlustAction();
		assertFalse(cmd.isChange());
	}

	@Test
	public void changeFalseStored() {
		ClientCommandBloodlustAction cmd = new ClientCommandBloodlustAction(false);
		assertFalse(cmd.isChange());
	}

	@Test
	public void getIdIsClientBloodlustAction() {
		assertEquals(NetCommandId.CLIENT_BLOODLUST_ACTION, new ClientCommandBloodlustAction(false).getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndChangeToMove() {
		ClientCommandBloodlustAction cmd = new ClientCommandBloodlustAction(true);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientBloodlustAction", json.get("netCommandId").asString());
		assertTrue(json.get("changeToMove").asBoolean());
	}

	@Test
	public void roundTripWithChangeTrueAndEntropy() {
		ClientCommandBloodlustAction cmd = new ClientCommandBloodlustAction(true);
		cmd.setEntropy((byte) 5);
		JsonObject json = cmd.toJsonValue();
		ClientCommand restored = new ClientCommandBloodlustAction().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 5, restored.getEntropy());
		assertTrue(((ClientCommandBloodlustAction) restored).isChange());
	}

	@Test
	public void roundTripWithDefault() {
		ClientCommandBloodlustAction cmd = new ClientCommandBloodlustAction();
		JsonObject json = cmd.toJsonValue();
		ClientCommand restored = new ClientCommandBloodlustAction().initFrom(NetCommandTestUtil.gameSource(), json);
		assertFalse(restored.hasEntropy());
		assertFalse(((ClientCommandBloodlustAction) restored).isChange());
	}
}
