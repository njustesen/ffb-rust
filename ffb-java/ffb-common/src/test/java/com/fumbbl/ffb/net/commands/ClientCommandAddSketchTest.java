package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.model.sketch.Sketch;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_add_sketch.rs tests.
 *
 * Note: Java's {@link Sketch} generates its id as a random UUID and exposes no setter, so the
 * Rust tests that assert a caller-supplied id ("sk-123", "sk-9") are adapted to assert against
 * the sketch's own generated id (self-consistent). The Rust round_trip_with_default_data test is
 * SKIPPED: Java's ClientCommandAddSketch.toJsonValue() dereferences the sketch unconditionally
 * and NPEs when it is null, so a default (no-sketch) command cannot be serialized.
 */
public class ClientCommandAddSketchTest {

	@Test
	public void sketchIdStored() {
		Sketch sketch = new Sketch(0);
		ClientCommandAddSketch cmd = new ClientCommandAddSketch(sketch);
		assertEquals(sketch.getId(), cmd.getSketch().getId());
	}

	@Test
	public void defaultIsNone() {
		ClientCommandAddSketch cmd = new ClientCommandAddSketch();
		assertNull(cmd.getSketch());
	}

	@Test
	public void withSketchCarriesRgbLabelAndPath() {
		Sketch sketch = new Sketch(255);
		sketch.setLabel("note");
		sketch.addCoordinate(new FieldCoordinate(1, 2));
		ClientCommandAddSketch cmd = new ClientCommandAddSketch(sketch);
		assertEquals(sketch.getId(), cmd.getSketch().getId());
		assertEquals(255, cmd.getSketch().getRgb());
		assertEquals("note", cmd.getSketch().getLabel());
		assertEquals(1, cmd.getSketch().getPath().size());
		assertEquals(new FieldCoordinate(1, 2), cmd.getSketch().getPath().get(0));
	}

	@Test
	public void roundTripPreservesRgbLabelAndPath() {
		Sketch sketch = new Sketch(16);
		sketch.setLabel("lbl");
		sketch.addCoordinate(new FieldCoordinate(5, 6));
		String id = sketch.getId();
		ClientCommandAddSketch cmd = new ClientCommandAddSketch(sketch);
		JsonObject json = cmd.toJsonValue();
		ClientCommandAddSketch restored = new ClientCommandAddSketch().initFrom(NetCommandTestUtil.gameSource(), json);
		Sketch restoredSketch = restored.getSketch();
		assertEquals(id, restoredSketch.getId());
		assertEquals(16, restoredSketch.getRgb());
		assertEquals("lbl", restoredSketch.getLabel());
		assertEquals(1, restoredSketch.getPath().size());
		assertEquals(new FieldCoordinate(5, 6), restoredSketch.getPath().get(0));
	}

	@Test
	public void getIdIsClientAddSketch() {
		assertEquals(NetCommandId.CLIENT_ADD_SKETCH, new ClientCommandAddSketch().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndSketchId() {
		Sketch sketch = new Sketch(0);
		ClientCommandAddSketch cmd = new ClientCommandAddSketch(sketch);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientAddSketch", json.get("netCommandId").asString());
		assertEquals(sketch.getId(), json.get("sketch").asObject().get("id").asString());
	}

	@Test
	public void roundTripWithSketchIdAndEntropy() {
		Sketch sketch = new Sketch(0);
		String id = sketch.getId();
		ClientCommandAddSketch cmd = new ClientCommandAddSketch(sketch);
		cmd.setEntropy((byte) 4);
		JsonObject json = cmd.toJsonValue();
		ClientCommandAddSketch restored = new ClientCommandAddSketch().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 4, restored.getEntropy());
		assertEquals(id, restored.getSketch().getId());
	}
}
