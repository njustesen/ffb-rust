package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.Pushback;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_pushback.rs tests.
 *
 * The Rust {@code round_trip_default} test is inexpressible in Java: the Java command's
 * {@code toJsonValue()} unconditionally dereferences {@code fPushback.toJsonValue()}, and the
 * default (no-arg) command has a null pushback, so serializing it throws NPE. That test is skipped.
 */
public class ClientCommandPushbackTest {

	@Test
	public void pushbackStored() {
		Pushback p = new Pushback("p1", new FieldCoordinate(3, 5));
		ClientCommandPushback cmd = new ClientCommandPushback(p);
		assertEquals("p1", cmd.getPushback().getPlayerId());
	}

	@Test
	public void defaultNone() {
		ClientCommandPushback cmd = new ClientCommandPushback();
		assertNull(cmd.getPushback());
	}

	@Test
	public void coordinateAccessibleViaPushback() {
		FieldCoordinate coord = new FieldCoordinate(7, 2);
		Pushback p = new Pushback("p2", coord);
		ClientCommandPushback cmd = new ClientCommandPushback(p);
		assertEquals(coord, cmd.getPushback().getCoordinate());
	}

	@Test
	public void getIdIsClientPushback() {
		assertEquals(NetCommandId.CLIENT_PUSHBACK, new ClientCommandPushback().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndPushback() {
		Pushback p = new Pushback("p1", new FieldCoordinate(2, 9));
		ClientCommandPushback cmd = new ClientCommandPushback(p);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientPushback", json.get("netCommandId").asString());
		JsonObject pushback = json.get("pushback").asObject();
		assertEquals("p1", pushback.get("playerId").asString());
		assertEquals(2, pushback.get("coordinate").asArray().get(0).asInt());
		assertEquals(9, pushback.get("coordinate").asArray().get(1).asInt());
	}

	@Test
	public void roundTripWithData() {
		Pushback p = new Pushback("p5", new FieldCoordinate(5, 6));
		ClientCommandPushback cmd = new ClientCommandPushback(p);
		cmd.setEntropy((byte) 4);
		JsonObject json = cmd.toJsonValue();
		ClientCommandPushback restored = new ClientCommandPushback().initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 4, restored.getEntropy());
		assertEquals("p5", restored.getPushback().getPlayerId());
		assertEquals(new FieldCoordinate(5, 6), restored.getPushback().getCoordinate());
	}
}
