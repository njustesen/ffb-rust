package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/server_command_team_setup_list.rs tests.
 * Note: the Rust add_name test exercises add_setup_name, which is private in Java; the
 * equivalent single-name addition is expressed through the String[] constructor.
 */
public class ServerCommandTeamSetupListTest {

	@Test
	public void fieldsStored() {
		ServerCommandTeamSetupList cmd = new ServerCommandTeamSetupList(new String[] { "Wide", "Cage" });
		assertArrayEquals(new String[] { "Wide", "Cage" }, cmd.getSetupNames());
	}

	@Test
	public void addName() {
		ServerCommandTeamSetupList cmd = new ServerCommandTeamSetupList(new String[] { "Press" });
		assertEquals(1, cmd.getSetupNames().length);
		assertEquals("Press", cmd.getSetupNames()[0]);
	}

	@Test
	public void getIdIsServerTeamSetupList() {
		assertEquals(NetCommandId.SERVER_TEAM_SETUP_LIST, new ServerCommandTeamSetupList().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndSetupNames() {
		ServerCommandTeamSetupList cmd = new ServerCommandTeamSetupList(new String[] { "Wide" });
		JsonObject json = cmd.toJsonValue();
		assertEquals("serverTeamSetupList", json.get("netCommandId").asString());
		assertEquals("Wide", json.get("setupNames").asArray().get(0).asString());
	}

	@Test
	public void roundTripWithNames() {
		ServerCommandTeamSetupList cmd = new ServerCommandTeamSetupList(new String[] { "Wide", "Cage" });
		cmd.setCommandNr(6);
		JsonObject json = cmd.toJsonValue();
		ServerCommandTeamSetupList restored = new ServerCommandTeamSetupList().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(6, restored.getCommandNr());
		assertArrayEquals(new String[] { "Wide", "Cage" }, restored.getSetupNames());
	}

	@Test
	public void roundTripWithNoNames() {
		ServerCommandTeamSetupList cmd = new ServerCommandTeamSetupList();
		JsonObject json = cmd.toJsonValue();
		ServerCommandTeamSetupList restored = new ServerCommandTeamSetupList().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(0, restored.getSetupNames().length);
	}
}
