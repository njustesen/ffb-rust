package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/server_command_sketch_add_coordinate.rs tests.
 */
public class ServerCommandSketchAddCoordinateTest {

	@Test
	public void fieldsStored() {
		ServerCommandSketchAddCoordinate cmd = new ServerCommandSketchAddCoordinate("Alice", "sk1", new FieldCoordinate(5, 3));
		assertEquals("Alice", cmd.getCoach());
		assertEquals("sk1", cmd.getSketchId());
		assertEquals(new FieldCoordinate(5, 3), cmd.getCoordinate());
	}

	@Test
	public void getIdIsServerSketchAddCoordinate() {
		assertEquals(NetCommandId.SERVER_SKETCH_ADD_COORDINATE, new ServerCommandSketchAddCoordinate().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndIdKey() {
		ServerCommandSketchAddCoordinate cmd = new ServerCommandSketchAddCoordinate("Alice", "sk1", new FieldCoordinate(5, 3));
		JsonObject json = cmd.toJsonValue();
		assertEquals("serverSketchAddCoordinate", json.get("netCommandId").asString());
		assertEquals("sk1", json.get("id").asString());
		assertEquals("Alice", json.get("coach").asString());
		assertEquals(5, json.get("coordinate").asArray().get(0).asInt());
		assertEquals(3, json.get("coordinate").asArray().get(1).asInt());
	}

	@Test
	public void roundTripWithData() {
		ServerCommandSketchAddCoordinate cmd = new ServerCommandSketchAddCoordinate("Bob", "sk2", new FieldCoordinate(1, 2));
		JsonObject json = cmd.toJsonValue();
		ServerCommandSketchAddCoordinate restored = new ServerCommandSketchAddCoordinate().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals("Bob", restored.getCoach());
		assertEquals("sk2", restored.getSketchId());
		assertEquals(new FieldCoordinate(1, 2), restored.getCoordinate());
	}

	@Test
	public void roundTripWithDefault() {
		ServerCommandSketchAddCoordinate cmd = new ServerCommandSketchAddCoordinate();
		JsonObject json = cmd.toJsonValue();
		ServerCommandSketchAddCoordinate restored = new ServerCommandSketchAddCoordinate().initFrom(NetCommandTestUtil.gameSource(), json);
		// Java leaves coach/sketchId null and coordinate null (Rust default() is empty strings and (0,0)).
		assertNull(restored.getCoach());
		assertNull(restored.getSketchId());
		assertNull(restored.getCoordinate());
	}
}
