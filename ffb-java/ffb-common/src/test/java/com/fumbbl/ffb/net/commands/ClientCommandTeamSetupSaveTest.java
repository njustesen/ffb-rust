package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_team_setup_save.rs tests.
 */
public class ClientCommandTeamSetupSaveTest {

	@Test
	public void fieldsStored() {
		FieldCoordinate[] coords = new FieldCoordinate[]{new FieldCoordinate(1, 2), new FieldCoordinate(3, 4)};
		ClientCommandTeamSetupSave cmd = new ClientCommandTeamSetupSave("default", new int[]{1, 2}, coords);
		assertEquals("default", cmd.getSetupName());
		assertArrayEquals(new int[]{1, 2}, cmd.getPlayerNumbers());
		assertEquals(2, cmd.getPlayerCoordinates().length);
	}

	@Test
	public void defaultIsEmpty() {
		ClientCommandTeamSetupSave cmd = new ClientCommandTeamSetupSave();
		assertNull(cmd.getSetupName());
		assertEquals(0, cmd.getPlayerNumbers().length);
		assertEquals(0, cmd.getPlayerCoordinates().length);
	}

	@Test
	public void getIdIsClientTeamSetupSave() {
		assertEquals(NetCommandId.CLIENT_TEAM_SETUP_SAVE, new ClientCommandTeamSetupSave().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndPlayerNumbers() {
		ClientCommandTeamSetupSave cmd = new ClientCommandTeamSetupSave("s1", new int[]{1, 2, 3}, new FieldCoordinate[]{});
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientTeamSetupSave", json.get("netCommandId").asString());
		assertEquals(3, json.get("playerNumbers").asArray().size());
		assertEquals(1, json.get("playerNumbers").asArray().get(0).asInt());
		assertEquals(2, json.get("playerNumbers").asArray().get(1).asInt());
		assertEquals(3, json.get("playerNumbers").asArray().get(2).asInt());
	}

	@Test
	public void roundTripWithAllFieldsAndEntropy() {
		FieldCoordinate[] coords = new FieldCoordinate[]{new FieldCoordinate(5, 6), new FieldCoordinate(7, 8)};
		ClientCommandTeamSetupSave cmd = new ClientCommandTeamSetupSave("setup-a", new int[]{4, 9}, coords);
		cmd.setEntropy((byte) 13);
		JsonObject json = cmd.toJsonValue();
		ClientCommandTeamSetupSave restored = new ClientCommandTeamSetupSave().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 13, restored.getEntropy());
		assertEquals("setup-a", restored.getSetupName());
		assertArrayEquals(new int[]{4, 9}, restored.getPlayerNumbers());
		assertArrayEquals(coords, restored.getPlayerCoordinates());
	}

	@Test
	public void roundTripWithEmptyDefaults() {
		ClientCommandTeamSetupSave cmd = new ClientCommandTeamSetupSave();
		JsonObject json = cmd.toJsonValue();
		ClientCommandTeamSetupSave restored = new ClientCommandTeamSetupSave().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getSetupName());
		assertEquals(0, restored.getPlayerNumbers().length);
		assertEquals(0, restored.getPlayerCoordinates().length);
	}
}
