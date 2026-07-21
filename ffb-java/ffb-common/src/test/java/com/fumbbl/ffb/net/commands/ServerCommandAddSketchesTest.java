package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.model.sketch.Sketch;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import java.util.ArrayList;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/server_command_add_sketches.rs tests.
 */
public class ServerCommandAddSketchesTest {

	@Test
	public void fieldsStored() {
		List<Sketch> sketches = new ArrayList<>();
		sketches.add(new Sketch(0));
		ServerCommandAddSketches cmd = new ServerCommandAddSketches("Alice", sketches);
		assertEquals("Alice", cmd.getCoach());
		assertEquals(1, cmd.getSketches().size());
	}

	@Test
	public void defaultEmpty() {
		ServerCommandAddSketches cmd = new ServerCommandAddSketches();
		assertNull(cmd.getCoach());
		assertTrue(cmd.getSketches().isEmpty());
	}

	@Test
	public void getIdIsServerAddSketches() {
		assertEquals(NetCommandId.SERVER_ADD_SKETCHES, new ServerCommandAddSketches().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndCoachAndNoCommandNr() {
		ServerCommandAddSketches cmd = new ServerCommandAddSketches("Alice", new ArrayList<>());
		JsonObject json = cmd.toJsonValue();
		assertEquals("serverAddSketches", json.get("netCommandId").asString());
		assertEquals("Alice", json.get("coach").asString());
		assertNull(json.get("commandNr"));
	}

	@Test
	public void roundTripWithSketches() {
		Sketch sketch = new Sketch(0);
		sketch.addCoordinate(new FieldCoordinate(2, 3));
		List<Sketch> sketches = new ArrayList<>();
		sketches.add(sketch);
		ServerCommandAddSketches cmd = new ServerCommandAddSketches("Bob", sketches);
		JsonObject json = cmd.toJsonValue();
		ServerCommandAddSketches restored = new ServerCommandAddSketches().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals("Bob", restored.getCoach());
		assertEquals(1, restored.getSketches().size());
		assertEquals(1, restored.getSketches().get(0).getPath().size());
	}

	@Test
	public void roundTripWithEmptySketches() {
		ServerCommandAddSketches cmd = new ServerCommandAddSketches();
		JsonObject json = cmd.toJsonValue();
		ServerCommandAddSketches restored = new ServerCommandAddSketches().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getCoach());
		assertTrue(restored.getSketches().isEmpty());
	}
}
