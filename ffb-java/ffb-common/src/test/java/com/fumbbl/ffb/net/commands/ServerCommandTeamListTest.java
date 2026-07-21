package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.TeamList;
import com.fumbbl.ffb.TeamListEntry;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/server_command_team_list.rs tests.
 */
public class ServerCommandTeamListTest {

	@Test
	public void fieldsStored() {
		ServerCommandTeamList cmd = new ServerCommandTeamList(new TeamList());
		assertNotNull(cmd.getTeamList());
	}

	@Test
	public void getIdIsServerTeamList() {
		assertEquals(NetCommandId.SERVER_TEAM_LIST, new ServerCommandTeamList().getId());
	}

	@Test
	public void toJsonValueHasNetCommandId() {
		ServerCommandTeamList cmd = new ServerCommandTeamList();
		JsonObject json = cmd.toJsonValue();
		assertEquals("serverTeamList", json.get("netCommandId").asString());
		assertNull(json.get("teamList"));
	}

	@Test
	public void roundTripWithTeamList() {
		TeamListEntry entry = new TeamListEntry();
		entry.setTeamId("t1");
		entry.setTeamName("Orcs");
		entry.setRace("Orc");
		TeamList tl = new TeamList("Alice", new TeamListEntry[] { entry });
		ServerCommandTeamList cmd = new ServerCommandTeamList(tl);
		cmd.setCommandNr(2);
		JsonObject json = cmd.toJsonValue();
		ServerCommandTeamList restored = new ServerCommandTeamList().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(2, restored.getCommandNr());
		TeamList restoredList = restored.getTeamList();
		assertNotNull(restoredList);
		assertEquals("Alice", restoredList.getCoach());
		assertEquals(1, restoredList.getTeamListEntries().length);
		assertEquals("Orcs", restoredList.getTeamListEntries()[0].getTeamName());
	}

	@Test
	public void roundTripWithNoTeamList() {
		ServerCommandTeamList cmd = new ServerCommandTeamList();
		JsonObject json = cmd.toJsonValue();
		ServerCommandTeamList restored = new ServerCommandTeamList().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getTeamList());
	}
}
