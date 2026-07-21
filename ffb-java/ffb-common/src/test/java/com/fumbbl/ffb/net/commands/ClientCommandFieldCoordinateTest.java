package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_field_coordinate.rs tests.
 *
 * Wire-format note: Java's FieldCoordinate serializes to a JSON object {"x":..,"y":..},
 * not the [x,y] array the Rust test asserts. The JSON key assertions here use the
 * Java-correct object shape.
 */
public class ClientCommandFieldCoordinateTest {

	@Test
	public void defaultHasNoCoordinate() {
		ClientCommandFieldCoordinate cmd = new ClientCommandFieldCoordinate();
		assertNull(cmd.getFieldCoordinate());
	}

	@Test
	public void withCoordinateStoresValue() {
		ClientCommandFieldCoordinate cmd = new ClientCommandFieldCoordinate(new FieldCoordinate(3, 5));
		assertEquals(new FieldCoordinate(3, 5), cmd.getFieldCoordinate());
	}

	@Test
	public void getIdIsClientFieldCoordinate() {
		assertEquals(NetCommandId.CLIENT_FIELD_COORDINATE, new ClientCommandFieldCoordinate().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndCoordinate() {
		ClientCommandFieldCoordinate cmd = new ClientCommandFieldCoordinate(new FieldCoordinate(4, 8));
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientFieldCoordinate", json.get("netCommandId").asString());
		JsonObject coordinate = json.get("fieldCoordinate").asObject();
		assertEquals(4, coordinate.get("x").asInt());
		assertEquals(8, coordinate.get("y").asInt());
	}

	@Test
	public void roundTripWithCoordinateAndEntropy() {
		ClientCommandFieldCoordinate cmd = new ClientCommandFieldCoordinate(new FieldCoordinate(1, 2));
		cmd.setEntropy((byte) 11);
		JsonObject json = cmd.toJsonValue();
		ClientCommandFieldCoordinate restored = new ClientCommandFieldCoordinate().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 11, restored.getEntropy());
		assertEquals(new FieldCoordinate(1, 2), restored.getFieldCoordinate());
	}
}
