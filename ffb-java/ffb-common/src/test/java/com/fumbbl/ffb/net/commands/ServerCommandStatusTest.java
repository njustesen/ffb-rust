package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import com.fumbbl.ffb.net.ServerStatus;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/server_command_status.rs tests.
 */
public class ServerCommandStatusTest {

	@Test
	public void fieldsStored() {
		ServerCommandStatus cmd = new ServerCommandStatus(ServerStatus.FUMBBL_ERROR, "Connected");
		assertEquals(ServerStatus.FUMBBL_ERROR, cmd.getServerStatus());
		assertEquals("Connected", cmd.getMessage());
	}

	@Test
	public void defaultEmpty() {
		ServerCommandStatus cmd = new ServerCommandStatus();
		assertNull(cmd.getServerStatus());
	}

	@Test
	public void getIdIsServerStatus() {
		assertEquals(NetCommandId.SERVER_STATUS, new ServerCommandStatus().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndStatus() {
		ServerCommandStatus cmd = new ServerCommandStatus(ServerStatus.ERROR_WRONG_PASSWORD, "bad password");
		JsonObject json = cmd.toJsonValue();
		assertEquals("serverStatus", json.get("netCommandId").asString());
		assertEquals("Wrong Password", json.get("serverStatus").asString());
		assertEquals("bad password", json.get("message").asString());
	}

	@Test
	public void roundTripWithStatus() {
		ServerCommandStatus cmd = new ServerCommandStatus(ServerStatus.ERROR_GAME_IN_USE, "in use");
		cmd.setCommandNr(4);
		JsonObject json = cmd.toJsonValue();
		ServerCommandStatus restored = new ServerCommandStatus().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(4, restored.getCommandNr());
		assertEquals(ServerStatus.ERROR_GAME_IN_USE, restored.getServerStatus());
		assertEquals("in use", restored.getMessage());
	}

	@Test
	public void roundTripWithNoStatus() {
		ServerCommandStatus cmd = new ServerCommandStatus();
		JsonObject json = cmd.toJsonValue();
		ServerCommandStatus restored = new ServerCommandStatus().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getServerStatus());
		assertNull(restored.getMessage());
	}
}
