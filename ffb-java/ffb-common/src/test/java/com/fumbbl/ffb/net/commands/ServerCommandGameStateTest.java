package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.FactoryType.FactoryContext;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/server_command_game_state.rs tests.
 * The Rust `serde_round_trip_no_game` test exercises the Rust struct's serde derive; it is
 * adapted here to the Java toJsonValue/initFrom round-trip path (same intent: commandNr
 * survives with no game).
 */
public class ServerCommandGameStateTest {

	private Game buildGame() {
		Game game = new Game(NetCommandTestUtil.applicationSource(),
			NetCommandTestUtil.applicationSource().getFactoryManager());
		game.initializeRules();
		return game;
	}

	@Test
	public void notReplayable() {
		assertFalse(new ServerCommandGameState().isReplayable());
	}

	@Test
	public void idIsServerGameState() {
		assertEquals(NetCommandId.SERVER_GAME_STATE, new ServerCommandGameState().getId());
	}

	@Test
	public void newWithNoGame() {
		ServerCommandGameState cmd = new ServerCommandGameState();
		assertNull(cmd.getGame());
	}

	@Test
	public void serdeRoundTripNoGame() {
		ServerCommandGameState cmd = new ServerCommandGameState();
		cmd.setCommandNr(5);
		JsonObject json = cmd.toJsonValue();
		ServerCommandGameState restored = new ServerCommandGameState().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(5, restored.getCommandNr());
		assertNull(restored.getGame());
	}

	@Test
	public void getIdViaNetCommandTrait() {
		assertEquals(NetCommandId.SERVER_GAME_STATE, new ServerCommandGameState().getId());
	}

	@Test
	public void getContextIsApplication() {
		assertEquals(FactoryContext.APPLICATION, new ServerCommandGameState().getContext());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndNoGame() {
		ServerCommandGameState cmd = new ServerCommandGameState();
		cmd.setCommandNr(3);
		JsonObject json = cmd.toJsonValue();
		assertEquals("serverGameState", json.get("netCommandId").asString());
		assertEquals(3, json.get("commandNr").asInt());
		assertNull(json.get("game"));
	}

	@Test
	public void roundTripWithGame() {
		ServerCommandGameState cmd = new ServerCommandGameState(buildGame());
		cmd.setCommandNr(11);
		JsonObject json = cmd.toJsonValue();
		ServerCommandGameState restored = new ServerCommandGameState().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(11, restored.getCommandNr());
		assertNotNull(restored.getGame());
	}

	@Test
	public void roundTripWithNoGame() {
		ServerCommandGameState cmd = new ServerCommandGameState();
		JsonObject json = cmd.toJsonValue();
		ServerCommandGameState restored = new ServerCommandGameState().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getGame());
	}
}
