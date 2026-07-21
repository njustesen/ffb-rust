package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/server_command_unzap_player.rs tests.
 * Note: the Java constructor takes (playerId, teamId) — reversed relative to the Rust
 * new(team_id, player_id).
 * The Rust debug_format_works test exercises the Debug derive and has no Java equivalent; skipped.
 */
public class ServerCommandUnzapPlayerTest {

	@Test
	public void fieldsStored() {
		ServerCommandUnzapPlayer cmd = new ServerCommandUnzapPlayer("p1", "team1");
		assertEquals("team1", cmd.getTeamId());
		assertEquals("p1", cmd.getPlayerId());
	}

	@Test
	public void getIdIsServerUnzapPlayer() {
		assertEquals(NetCommandId.SERVER_UNZAP_PLAYER, new ServerCommandUnzapPlayer().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndIds() {
		ServerCommandUnzapPlayer cmd = new ServerCommandUnzapPlayer("p1", "team1");
		JsonObject json = cmd.toJsonValue().asObject();
		assertEquals("serverUnzapPlayer", json.get("netCommandId").asString());
		assertEquals("team1", json.get("teamId").asString());
		assertEquals("p1", json.get("playerId").asString());
	}

	@Test
	public void roundTripWithIds() {
		ServerCommandUnzapPlayer cmd = new ServerCommandUnzapPlayer("p1", "team1");
		cmd.setCommandNr(8);
		JsonObject json = cmd.toJsonValue().asObject();
		ServerCommandUnzapPlayer restored = (ServerCommandUnzapPlayer) new ServerCommandUnzapPlayer().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(8, restored.getCommandNr());
		assertEquals("team1", restored.getTeamId());
		assertEquals("p1", restored.getPlayerId());
	}

	@Test
	public void roundTripWithDefault() {
		ServerCommandUnzapPlayer cmd = new ServerCommandUnzapPlayer();
		JsonObject json = cmd.toJsonValue().asObject();
		ServerCommandUnzapPlayer restored = (ServerCommandUnzapPlayer) new ServerCommandUnzapPlayer().initFrom(NetCommandTestUtil.gameSource(), json);
		// Java restores null (Rust default() is empty string).
		assertNull(restored.getTeamId());
		assertNull(restored.getPlayerId());
	}
}
