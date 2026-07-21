package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.GameList;
import com.fumbbl.ffb.GameListEntry;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/server_command_game_list.rs tests.
 */
public class ServerCommandGameListTest {

	@Test
	public void fieldsStored() {
		ServerCommandGameList cmd = new ServerCommandGameList(new GameList());
		assertNotNull(cmd.getGameList());
	}

	@Test
	public void getIdIsServerGameList() {
		assertEquals(NetCommandId.SERVER_GAME_LIST, new ServerCommandGameList().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndGameList() {
		ServerCommandGameList cmd = new ServerCommandGameList(new GameList());
		cmd.setCommandNr(2);
		JsonObject json = cmd.toJsonValue();
		assertEquals("serverGameList", json.get("netCommandId").asString());
		assertEquals(2, json.get("commandNr").asInt());
		assertNotNull(json.get("gameList"));
	}

	@Test
	public void roundTripWithEntries() {
		GameList list = new GameList();
		list.add(new GameListEntry());
		ServerCommandGameList cmd = new ServerCommandGameList(list);
		cmd.setCommandNr(9);
		JsonObject json = cmd.toJsonValue();
		ServerCommandGameList restored = new ServerCommandGameList().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(9, restored.getCommandNr());
		assertEquals(1, restored.getGameList().size());
	}

	@Test
	public void roundTripWithEmptyGameList() {
		ServerCommandGameList cmd = new ServerCommandGameList(new GameList());
		JsonObject json = cmd.toJsonValue();
		ServerCommandGameList restored = new ServerCommandGameList().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNotNull(restored.getGameList());
		assertEquals(0, restored.getGameList().size());
	}
}
