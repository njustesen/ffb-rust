package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_team_setup_load.rs tests.
 */
public class ClientCommandTeamSetupLoadTest {

	@Test
	public void defaultHasNoSetupName() {
		ClientCommandTeamSetupLoad cmd = new ClientCommandTeamSetupLoad();
		assertNull(cmd.getSetupName());
	}

	@Test
	public void withSetupNameStoresValue() {
		ClientCommandTeamSetupLoad cmd = new ClientCommandTeamSetupLoad("default-setup");
		assertEquals("default-setup", cmd.getSetupName());
	}

	@Test
	public void getIdIsClientTeamSetupLoad() {
		assertEquals(NetCommandId.CLIENT_TEAM_SETUP_LOAD, new ClientCommandTeamSetupLoad().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndSetupName() {
		ClientCommandTeamSetupLoad cmd = new ClientCommandTeamSetupLoad("s1");
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientTeamSetupLoad", json.get("netCommandId").asString());
		assertEquals("s1", json.get("setupName").asString());
	}

	@Test
	public void roundTripWithSetupNameAndEntropy() {
		ClientCommandTeamSetupLoad cmd = new ClientCommandTeamSetupLoad("s2");
		cmd.setEntropy((byte) 11);
		JsonObject json = cmd.toJsonValue();
		ClientCommandTeamSetupLoad restored = new ClientCommandTeamSetupLoad().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 11, restored.getEntropy());
		assertEquals("s2", restored.getSetupName());
	}

	@Test
	public void roundTripWithNoSetupName() {
		ClientCommandTeamSetupLoad cmd = new ClientCommandTeamSetupLoad();
		JsonObject json = cmd.toJsonValue();
		ClientCommandTeamSetupLoad restored = new ClientCommandTeamSetupLoad().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getSetupName());
	}
}
