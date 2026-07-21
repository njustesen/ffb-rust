package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import java.util.Arrays;
import java.util.Collections;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/server_command_sketch_set_color.rs tests.
 * Note: the Rust round_trip_with_default test is not portable — the Java class leaves
 * sketchIds null on the no-arg constructor, and initFrom does Arrays.asList((String[]) null)
 * which throws NullPointerException. Skipped.
 */
public class ServerCommandSketchSetColorTest {

	@Test
	public void fieldsStored() {
		List<String> ids = Arrays.asList("s1", "s2");
		ServerCommandSketchSetColor cmd = new ServerCommandSketchSetColor("Bob", ids, 0xFF0000);
		assertEquals("Bob", cmd.getCoach());
		assertEquals(ids, cmd.getSketchIds());
		assertEquals(0xFF0000, cmd.getRbg());
	}

	@Test
	public void defaultEmpty() {
		ServerCommandSketchSetColor cmd = new ServerCommandSketchSetColor();
		// Java leaves coach and sketchIds null (Rust default() is empty string/vec).
		assertNull(cmd.getCoach());
		assertNull(cmd.getSketchIds());
		assertEquals(0, cmd.getRbg());
	}

	@Test
	public void getIdIsServerSketchSetColor() {
		assertEquals(NetCommandId.SERVER_SKETCH_SET_COLOR, new ServerCommandSketchSetColor().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndRgb() {
		ServerCommandSketchSetColor cmd = new ServerCommandSketchSetColor("Bob", Collections.singletonList("s1"), 0xFF0000);
		JsonObject json = cmd.toJsonValue();
		assertEquals("serverSketchSetColor", json.get("netCommandId").asString());
		assertEquals(0xFF0000, json.get("rgb").asInt());
		assertEquals("s1", json.get("ids").asArray().get(0).asString());
		assertEquals("Bob", json.get("coach").asString());
	}

	@Test
	public void roundTripWithData() {
		ServerCommandSketchSetColor cmd = new ServerCommandSketchSetColor("Carol", Arrays.asList("s1", "s2"), 42);
		JsonObject json = cmd.toJsonValue();
		ServerCommandSketchSetColor restored = new ServerCommandSketchSetColor().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals("Carol", restored.getCoach());
		assertEquals(Arrays.asList("s1", "s2"), restored.getSketchIds());
		assertEquals(42, restored.getRbg());
	}
}
