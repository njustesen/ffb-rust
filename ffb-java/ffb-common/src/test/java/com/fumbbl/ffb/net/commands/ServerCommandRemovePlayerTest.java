package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/server_command_remove_player.rs tests.
 */
public class ServerCommandRemovePlayerTest {

	@Test
	public void playerIdStored() {
		ServerCommandRemovePlayer cmd = new ServerCommandRemovePlayer("p1");
		assertEquals("p1", cmd.getPlayerId());
	}

	@Test
	public void defaultEmpty() {
		ServerCommandRemovePlayer cmd = new ServerCommandRemovePlayer();
		assertNull(cmd.getPlayerId());
	}

	@Test
	public void getIdIsServerRemovePlayer() {
		assertEquals(NetCommandId.SERVER_REMOVE_PLAYER, new ServerCommandRemovePlayer("p1").getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndPlayerId() {
		ServerCommandRemovePlayer cmd = new ServerCommandRemovePlayer("p9");
		cmd.setCommandNr(3);
		JsonObject json = cmd.toJsonValue();
		assertEquals("serverRemovePlayer", json.get("netCommandId").asString());
		assertEquals(3, json.get("commandNr").asInt());
		assertEquals("p9", json.get("playerId").asString());
	}

	@Test
	public void roundTripWithData() {
		ServerCommandRemovePlayer cmd = new ServerCommandRemovePlayer("p42");
		cmd.setCommandNr(7);
		JsonObject json = cmd.toJsonValue();
		ServerCommandRemovePlayer restored = new ServerCommandRemovePlayer().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(7, restored.getCommandNr());
		assertEquals("p42", restored.getPlayerId());
	}

	@Test
	public void roundTripWithDefault() {
		ServerCommandRemovePlayer cmd = new ServerCommandRemovePlayer();
		JsonObject json = cmd.toJsonValue();
		ServerCommandRemovePlayer restored = new ServerCommandRemovePlayer().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(0, restored.getCommandNr());
		assertNull(restored.getPlayerId());
	}
}
