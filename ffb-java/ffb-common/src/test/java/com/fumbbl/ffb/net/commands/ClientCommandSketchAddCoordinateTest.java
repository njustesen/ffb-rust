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
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_sketch_add_coordinate.rs tests.
 */
public class ClientCommandSketchAddCoordinateTest {

	@Test
	public void fieldsStored() {
		FieldCoordinate coord = new FieldCoordinate(2, 9);
		ClientCommandSketchAddCoordinate cmd = new ClientCommandSketchAddCoordinate("sketch1", coord);
		assertEquals("sketch1", cmd.getSketchId());
		assertEquals(coord, cmd.getCoordinate());
	}

	@Test
	public void defaultIsEmpty() {
		ClientCommandSketchAddCoordinate cmd = new ClientCommandSketchAddCoordinate();
		assertNull(cmd.getSketchId());
		assertNull(cmd.getCoordinate());
	}

	@Test
	public void getIdIsClientSketchAddCoordinate() {
		assertEquals(NetCommandId.CLIENT_SKETCH_ADD_COORDINATE, new ClientCommandSketchAddCoordinate().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndIdKey() {
		ClientCommandSketchAddCoordinate cmd = new ClientCommandSketchAddCoordinate("sketch1", new FieldCoordinate(2, 9));
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientSketchAddCoordinate", json.get("netCommandId").asString());
		assertEquals("sketch1", json.get("id").asString());
	}

	@Test
	public void roundTripWithData() {
		ClientCommandSketchAddCoordinate cmd = new ClientCommandSketchAddCoordinate("sketch2", new FieldCoordinate(4, 4));
		cmd.setEntropy((byte) 11);
		JsonObject json = cmd.toJsonValue();
		ClientCommandSketchAddCoordinate restored = new ClientCommandSketchAddCoordinate().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals("sketch2", restored.getSketchId());
		assertEquals(new FieldCoordinate(4, 4), restored.getCoordinate());
		assertEquals((byte) 11, restored.getEntropy());
	}

	@Test
	public void roundTripDefault() {
		ClientCommandSketchAddCoordinate cmd = new ClientCommandSketchAddCoordinate();
		JsonObject json = cmd.toJsonValue();
		ClientCommandSketchAddCoordinate restored = new ClientCommandSketchAddCoordinate().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getSketchId());
		assertNull(restored.getCoordinate());
		assertFalse(restored.hasEntropy());
	}
}
