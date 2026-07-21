package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_team_setup_delete.rs tests.
 */
public class ClientCommandTeamSetupDeleteTest {

	@Test
	public void defaultHasNoSetupName() {
		ClientCommandTeamSetupDelete cmd = new ClientCommandTeamSetupDelete();
		assertNull(cmd.getSetupName());
	}

	@Test
	public void withSetupNameStoresValue() {
		ClientCommandTeamSetupDelete cmd = new ClientCommandTeamSetupDelete("my-setup");
		assertEquals("my-setup", cmd.getSetupName());
	}

	@Test
	public void getIdIsClientTeamSetupDelete() {
		assertEquals(NetCommandId.CLIENT_TEAM_SETUP_DELETE, new ClientCommandTeamSetupDelete().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndSetupName() {
		ClientCommandTeamSetupDelete cmd = new ClientCommandTeamSetupDelete("s1");
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientTeamSetupDelete", json.get("netCommandId").asString());
		assertEquals("s1", json.get("setupName").asString());
	}

	@Test
	public void roundTripWithSetupNameAndEntropy() {
		ClientCommandTeamSetupDelete cmd = new ClientCommandTeamSetupDelete("s2");
		cmd.setEntropy((byte) 11);
		JsonObject json = cmd.toJsonValue();
		ClientCommandTeamSetupDelete restored = new ClientCommandTeamSetupDelete().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 11, restored.getEntropy());
		assertEquals("s2", restored.getSetupName());
	}

	@Test
	public void roundTripWithNoSetupName() {
		ClientCommandTeamSetupDelete cmd = new ClientCommandTeamSetupDelete();
		JsonObject json = cmd.toJsonValue();
		ClientCommandTeamSetupDelete restored = new ClientCommandTeamSetupDelete().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getSetupName());
	}
}
