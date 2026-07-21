package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_set_marker.rs tests.
 *
 * Note: Java has two mutually exclusive constructors — {@code (FieldCoordinate, text)}
 * and {@code (playerId, text)} — and no setters, so a marker can never carry
 * playerId AND coordinate simultaneously (as the Rust {@code with_marker} builder does).
 * The coordinate-bearing tests use the {@code (FieldCoordinate, text)} constructor and
 * assert playerId stays null.
 */
public class ClientCommandSetMarkerTest {

	@Test
	public void withMarkerStoresAllFields() {
		FieldCoordinate coord = new FieldCoordinate(4, 6);
		ClientCommandSetMarker cmd = new ClientCommandSetMarker(coord, "X");
		assertEquals(coord, cmd.getCoordinate());
		assertEquals("X", cmd.getText());
		// Java coordinate-constructor leaves playerId null (see class note).
		assertNull(cmd.getPlayerId());
	}

	@Test
	public void defaultIsEmpty() {
		ClientCommandSetMarker cmd = new ClientCommandSetMarker();
		assertNull(cmd.getPlayerId());
		assertNull(cmd.getCoordinate());
		assertNull(cmd.getText());
	}

	@Test
	public void getIdIsClientSetMarker() {
		assertEquals(NetCommandId.CLIENT_SET_MARKER, new ClientCommandSetMarker().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndText() {
		ClientCommandSetMarker cmd = new ClientCommandSetMarker(new FieldCoordinate(4, 6), "X");
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientSetMarker", json.get("netCommandId").asString());
		assertEquals("X", json.get("text").asString());
		assertEquals(4, json.get("coordinate").asArray().get(0).asInt());
		assertEquals(6, json.get("coordinate").asArray().get(1).asInt());
	}

	@Test
	public void roundTripPopulated() {
		ClientCommandSetMarker cmd = new ClientCommandSetMarker(new FieldCoordinate(4, 6), "X");
		cmd.setEntropy((byte) 5);
		JsonObject json = cmd.toJsonValue();
		ClientCommandSetMarker restored = new ClientCommandSetMarker().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(new FieldCoordinate(4, 6), restored.getCoordinate());
		assertEquals("X", restored.getText());
		assertNull(restored.getPlayerId());
		assertEquals((byte) 5, restored.getEntropy());
	}

	@Test
	public void roundTripDefault() {
		ClientCommandSetMarker cmd = new ClientCommandSetMarker();
		JsonObject json = cmd.toJsonValue();
		ClientCommandSetMarker restored = new ClientCommandSetMarker().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getPlayerId());
		assertNull(restored.getCoordinate());
		assertNull(restored.getText());
		assertFalse(restored.hasEntropy());
	}
}
