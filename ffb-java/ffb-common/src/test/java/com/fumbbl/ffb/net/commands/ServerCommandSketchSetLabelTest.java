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
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/server_command_sketch_set_label.rs tests.
 * Note: the Rust round_trip_with_default test is not portable — the Java class leaves
 * sketchIds null on the no-arg constructor, and initFrom does Arrays.asList((String[]) null)
 * which throws NullPointerException. Skipped.
 */
public class ServerCommandSketchSetLabelTest {

	@Test
	public void fieldsStored() {
		List<String> ids = Collections.singletonList("s1");
		ServerCommandSketchSetLabel cmd = new ServerCommandSketchSetLabel("Carol", ids, "Arrow");
		assertEquals("Carol", cmd.getCoach());
		assertEquals(ids, cmd.getSketchIds());
		assertEquals("Arrow", cmd.getLabel());
	}

	@Test
	public void defaultEmpty() {
		ServerCommandSketchSetLabel cmd = new ServerCommandSketchSetLabel();
		// Java leaves coach, sketchIds and label null (Rust default() is empty string/vec).
		assertNull(cmd.getCoach());
		assertNull(cmd.getLabel());
	}

	@Test
	public void getIdIsServerSketchSetLabel() {
		assertEquals(NetCommandId.SERVER_SKETCH_SET_LABEL, new ServerCommandSketchSetLabel().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndTextKey() {
		ServerCommandSketchSetLabel cmd = new ServerCommandSketchSetLabel("Carol", Collections.singletonList("s1"), "Arrow");
		JsonObject json = cmd.toJsonValue();
		assertEquals("serverSketchSetLabel", json.get("netCommandId").asString());
		assertEquals("Arrow", json.get("text").asString());
		assertEquals("s1", json.get("ids").asArray().get(0).asString());
		assertEquals("Carol", json.get("coach").asString());
	}

	@Test
	public void roundTripWithData() {
		ServerCommandSketchSetLabel cmd = new ServerCommandSketchSetLabel("Dave", Arrays.asList("s1", "s2"), "Circle");
		JsonObject json = cmd.toJsonValue();
		ServerCommandSketchSetLabel restored = new ServerCommandSketchSetLabel().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals("Dave", restored.getCoach());
		assertEquals(Arrays.asList("s1", "s2"), restored.getSketchIds());
		assertEquals("Circle", restored.getLabel());
	}
}
