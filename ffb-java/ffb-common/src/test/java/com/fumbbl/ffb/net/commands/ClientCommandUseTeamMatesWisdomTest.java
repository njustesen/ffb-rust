package com.fumbbl.ffb.net.commands;

import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_use_team_mates_wisdom.rs tests.
 */
public class ClientCommandUseTeamMatesWisdomTest {

	@Test
	public void getIdIsClientUseTeamMatesWisdom() {
		assertEquals(NetCommandId.CLIENT_USE_TEAM_MATES_WISDOM, new ClientCommandUseTeamMatesWisdom().getId());
	}

	@Test
	public void toJsonValueHasNetCommandId() {
		JsonObject json = new ClientCommandUseTeamMatesWisdom().toJsonValue();
		assertEquals("clientUseTeamMatesWisdom", json.get("netCommandId").asString());
	}

	@Test
	public void roundTripWithEntropy() {
		ClientCommandUseTeamMatesWisdom cmd = new ClientCommandUseTeamMatesWisdom();
		cmd.setEntropy((byte) 2);
		JsonObject json = cmd.toJsonValue();
		ClientCommandUseTeamMatesWisdom restored = (ClientCommandUseTeamMatesWisdom) new ClientCommandUseTeamMatesWisdom()
			.initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 2, restored.getEntropy());
	}

	@Test
	public void roundTripWithNoEntropy() {
		ClientCommandUseTeamMatesWisdom cmd = new ClientCommandUseTeamMatesWisdom();
		JsonObject json = cmd.toJsonValue();
		ClientCommandUseTeamMatesWisdom restored = (ClientCommandUseTeamMatesWisdom) new ClientCommandUseTeamMatesWisdom()
			.initFrom(NetCommandTestUtil.gameSource(), json);
		assertFalse(restored.hasEntropy());
	}
}
