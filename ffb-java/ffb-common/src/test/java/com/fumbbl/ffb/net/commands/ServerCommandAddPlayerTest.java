package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.SendToBoxReason;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.GameResult;
import com.fumbbl.ffb.model.PlayerResult;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/server_command_add_player.rs tests.
 */
public class ServerCommandAddPlayerTest {

	private PlayerResult boxResult(SendToBoxReason reason, int turn, int half) {
		Game game = new Game(NetCommandTestUtil.applicationSource(),
			NetCommandTestUtil.applicationSource().getFactoryManager());
		GameResult gameResult = new GameResult(game);
		PlayerResult playerResult = new PlayerResult(gameResult.getTeamResultHome());
		playerResult.setSendToBoxReason(reason);
		playerResult.setSendToBoxTurn(turn);
		playerResult.setSendToBoxHalf(half);
		return playerResult;
	}

	@Test
	public void fieldsStored() {
		PlayerResult result = boxResult(SendToBoxReason.FOUL_BAN, 3, 0);
		ServerCommandAddPlayer cmd = new ServerCommandAddPlayer("team1", new RosterPlayer(), new PlayerState(0), result);
		assertEquals("team1", cmd.getTeamId());
		assertEquals(SendToBoxReason.FOUL_BAN, cmd.getSendToBoxReason());
		assertEquals(3, cmd.getSendToBoxTurn());
	}

	@Test
	public void defaultNoBox() {
		ServerCommandAddPlayer cmd = new ServerCommandAddPlayer();
		assertNull(cmd.getTeamId());
		assertNull(cmd.getSendToBoxReason());
	}

	@Test
	public void getIdIsServerAddPlayer() {
		assertEquals(NetCommandId.SERVER_ADD_PLAYER, new ServerCommandAddPlayer().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndTeamId() {
		ServerCommandAddPlayer cmd = new ServerCommandAddPlayer("team1", new RosterPlayer(), new PlayerState(0), null);
		cmd.setCommandNr(3);
		JsonObject json = cmd.toJsonValue();
		assertEquals("serverAddPlayer", json.get("netCommandId").asString());
		assertEquals(3, json.get("commandNr").asInt());
		assertEquals("team1", json.get("teamId").asString());
		// Discrepancy vs Rust: Java writes a null enum as a present JSON null key,
		// while the Rust translation omits sendToBoxReason entirely when None.
		assertTrue(json.get("sendToBoxReason").isNull());
	}

	@Test
	public void roundTripWithSendToBox() {
		PlayerResult result = boxResult(SendToBoxReason.FOULED, 2, 1);
		ServerCommandAddPlayer cmd = new ServerCommandAddPlayer("team1", new RosterPlayer(), new PlayerState(0), result);
		cmd.setCommandNr(5);
		JsonObject json = cmd.toJsonValue();
		ServerCommandAddPlayer restored = new ServerCommandAddPlayer().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(5, restored.getCommandNr());
		assertEquals("team1", restored.getTeamId());
		assertEquals(SendToBoxReason.FOULED, restored.getSendToBoxReason());
		assertEquals(2, restored.getSendToBoxTurn());
		assertEquals(1, restored.getSendToBoxHalf());
	}

	@Test
	public void roundTripDefaults() {
		ServerCommandAddPlayer cmd = new ServerCommandAddPlayer();
		JsonObject json = cmd.toJsonValue();
		ServerCommandAddPlayer restored = new ServerCommandAddPlayer().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getTeamId());
		assertNull(restored.getSendToBoxReason());
		assertEquals(0, restored.getSendToBoxTurn());
		assertEquals(0, restored.getSendToBoxHalf());
	}
}
