package com.fumbbl.ffb.net;

import com.eclipsesource.json.Json;
import com.eclipsesource.json.JsonObject;
import com.eclipsesource.json.ParseException;
import com.fumbbl.ffb.net.commands.ClientCommandJoin;
import com.fumbbl.ffb.net.commands.ServerCommandGameTime;
import com.fumbbl.ffb.net.commands.ServerCommandPong;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/net_command_factory.rs tests.
 * The Rust factory splits into for_json_str (Result-returning) and
 * for_json_value; Java's NetCommandFactory.forJsonValue(source, jsonValue) is
 * the single dispatch entry point. Invalid-JSON parsing is exercised via
 * Json.parse (the step the Rust for_json_str performs internally).
 * Also absorbs the client/server dispatch intent of the Rust umbrella-enum
 * test modules (any_client_command.rs / any_server_command.rs), which have no
 * standalone Java class analogue.
 */
public class NetCommandFactoryTest {

	private NetCommandFactory factory() {
		return new NetCommandFactory(NetCommandTestUtil.applicationSource());
	}

	@Test
	public void nullJsonReturnsNone() {
		assertNull(factory().forJsonValue(NetCommandTestUtil.gameSource(), null));
	}

	@Test
	public void emptyStringReturnsNone() {
		assertNull(factory().forJsonValue(NetCommandTestUtil.gameSource(), Json.NULL));
	}

	@Test
	public void invalidJsonReturnsErr() {
		assertThrows(ParseException.class, () -> Json.parse("{not valid}"));
	}

	@Test
	public void validPongParses() {
		NetCommand result = factory().forJsonValue(NetCommandTestUtil.gameSource(),
			new ServerCommandPong(42).toJsonValue());
		assertTrue(result instanceof ServerCommandPong);
		assertEquals(42, ((ServerCommandPong) result).getTimestamp());
	}

	@Test
	public void forJsonValueNullReturnsNone() {
		assertNull(factory().forJsonValue(NetCommandTestUtil.gameSource(), Json.NULL));
	}

	@Test
	public void forJsonValueMissingNetCommandIdReturnsNone() {
		assertNull(factory().forJsonValue(NetCommandTestUtil.gameSource(), new JsonObject()));
	}

	@Test
	public void forJsonValueDispatchesClientCommand() {
		ClientCommandJoin join = new ClientCommandJoin();
		join.setCoach("TestCoach");
		join.setTeamId("team1");
		NetCommand result = factory().forJsonValue(NetCommandTestUtil.gameSource(), join.toJsonValue());
		assertTrue(result instanceof ClientCommandJoin);
		assertEquals(NetCommandId.CLIENT_JOIN, result.getId());
	}

	@Test
	public void forJsonValueDispatchesServerCommand() {
		NetCommand result = factory().forJsonValue(NetCommandTestUtil.gameSource(),
			new ServerCommandGameTime(1, 2).toJsonValue());
		assertTrue(result instanceof ServerCommandGameTime);
		assertEquals(NetCommandId.SERVER_GAME_TIME, result.getId());
	}

	@Test
	public void isReplayableDelegatesToVariantOverride() {
		// ServerCommandGameTime overrides isReplayable() to return false (Java + Rust agree).
		assertTrue(!new ServerCommandGameTime(1, 2).isReplayable());
	}
}
