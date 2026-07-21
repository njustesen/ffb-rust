package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/server_command_zap_player.rs tests.
 * Note: the Java constructor takes (playerId, teamId) — reversed relative to the Rust
 * new(team_id, player_id).
 */
public class ServerCommandZapPlayerTest {

	@Test
	public void fieldsStored() {
		ServerCommandZapPlayer cmd = new ServerCommandZapPlayer("p5", "team2");
		assertEquals("team2", cmd.getTeamId());
		assertEquals("p5", cmd.getPlayerId());
	}

	@Test
	public void defaultEmpty() {
		ServerCommandZapPlayer cmd = new ServerCommandZapPlayer();
		// Java leaves playerId null (Rust default() is empty string).
		assertNull(cmd.getPlayerId());
	}

	@Test
	public void getIdIsServerZapPlayer() {
		assertEquals(NetCommandId.SERVER_ZAP_PLAYER, new ServerCommandZapPlayer().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndIds() {
		ServerCommandZapPlayer cmd = new ServerCommandZapPlayer("p5", "team2");
		JsonObject json = cmd.toJsonValue().asObject();
		assertEquals("serverZapPlayer", json.get("netCommandId").asString());
		assertEquals("team2", json.get("teamId").asString());
		assertEquals("p5", json.get("playerId").asString());
	}

	@Test
	public void roundTripWithIds() {
		ServerCommandZapPlayer cmd = new ServerCommandZapPlayer("p5", "team2");
		cmd.setCommandNr(9);
		JsonObject json = cmd.toJsonValue().asObject();
		ServerCommandZapPlayer restored = (ServerCommandZapPlayer) new ServerCommandZapPlayer().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(9, restored.getCommandNr());
		assertEquals("team2", restored.getTeamId());
		assertEquals("p5", restored.getPlayerId());
	}

	@Test
	public void roundTripWithDefault() {
		ServerCommandZapPlayer cmd = new ServerCommandZapPlayer();
		JsonObject json = cmd.toJsonValue().asObject();
		ServerCommandZapPlayer restored = (ServerCommandZapPlayer) new ServerCommandZapPlayer().initFrom(NetCommandTestUtil.gameSource(), json);
		// Java restores null (Rust default() is empty string).
		assertNull(restored.getTeamId());
		assertNull(restored.getPlayerId());
	}
}
