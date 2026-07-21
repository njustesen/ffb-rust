package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.factory.IFactorySource;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_load_automatic_player_markings.rs tests.
 *
 * Java differences from the Rust source:
 * - The Java command's {@code toJsonValue()} unconditionally calls {@code game.toJsonValue()},
 *   so a command whose game is null cannot be serialized (NPE). Rust models game as an Option
 *   and simply omits it. Serialization-based tests therefore supply a real Game, and the
 *   Rust {@code round_trip_default} test (default object, game None) is inexpressible in Java
 *   and is skipped.
 * - On the round trip, Java reconstructs a non-null Game, whereas the Rust test asserts the
 *   restored game is None; the assertion below is written to the Java-correct behavior.
 */
public class ClientCommandLoadAutomaticPlayerMarkingsTest {

	private static Game newGame() {
		IFactorySource app = NetCommandTestUtil.applicationSource();
		Game game = new Game(app, app.getFactoryManager());
		game.initializeRules();
		return game;
	}

	@Test
	public void defaultIndexIsZero() {
		ClientCommandLoadAutomaticPlayerMarkings cmd = new ClientCommandLoadAutomaticPlayerMarkings();
		assertEquals(0, cmd.getIndex());
	}

	@Test
	public void storesIndexAndCoach() {
		ClientCommandLoadAutomaticPlayerMarkings cmd =
			new ClientCommandLoadAutomaticPlayerMarkings(3, null, "CoachB");
		assertEquals(3, cmd.getIndex());
		assertEquals("CoachB", cmd.getCoach());
	}

	@Test
	public void coachNoneByDefault() {
		ClientCommandLoadAutomaticPlayerMarkings cmd = new ClientCommandLoadAutomaticPlayerMarkings();
		assertNull(cmd.getCoach());
	}

	@Test
	public void getIdIsClientLoadAutomaticPlayerMarkings() {
		assertEquals(NetCommandId.CLIENT_LOAD_AUTOMATIC_PLAYER_MARKINGS,
			new ClientCommandLoadAutomaticPlayerMarkings().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndSelectedIndex() {
		ClientCommandLoadAutomaticPlayerMarkings cmd =
			new ClientCommandLoadAutomaticPlayerMarkings(5, newGame(), "CoachB");
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientLoadPlayerMarkings", json.get("netCommandId").asString());
		assertEquals(5, json.get("selectedIndex").asInt());
		assertEquals("CoachB", json.get("coach").asString());
	}

	@Test
	public void roundTripWithIndexCoachAndEntropy() {
		ClientCommandLoadAutomaticPlayerMarkings cmd =
			new ClientCommandLoadAutomaticPlayerMarkings(2, newGame(), "CoachC");
		cmd.setEntropy((byte) 6);
		JsonObject json = cmd.toJsonValue();
		ClientCommandLoadAutomaticPlayerMarkings restored =
			(ClientCommandLoadAutomaticPlayerMarkings) new ClientCommandLoadAutomaticPlayerMarkings()
				.initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 6, restored.getEntropy());
		assertEquals(2, restored.getIndex());
		assertEquals("CoachC", restored.getCoach());
		// Java-correct: the game round-trips to a non-null Game (Rust asserts None here).
		assertNotNull(restored.getGame());
	}
}
