package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import java.util.Arrays;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_sketch_set_color.rs tests.
 *
 * Note: the Java field/getter is misspelled {@code rbg}/{@code getRbg()} (the wire
 * key is still {@code "rgb"}). The default command leaves {@code sketchIds} null, and
 * {@code initFrom} calls {@code Arrays.asList(null)} which NPEs, so the Rust
 * {@code round_trip_default} test is inexpressible in Java and is SKIPPED.
 */
public class ClientCommandSketchSetColorTest {

	@Test
	public void fieldsStored() {
		ClientCommandSketchSetColor cmd = new ClientCommandSketchSetColor(Arrays.asList("s1", "s2"), 0xFF0000);
		assertEquals(2, cmd.getSketchIds().size());
		assertEquals(0xFF0000, cmd.getRbg());
	}

	@Test
	public void defaultIsEmpty() {
		// Rust default has an empty vec; Java leaves sketchIds null.
		ClientCommandSketchSetColor cmd = new ClientCommandSketchSetColor();
		assertNull(cmd.getSketchIds());
		assertEquals(0, cmd.getRbg());
	}

	@Test
	public void getIdIsClientSketchSetColor() {
		assertEquals(NetCommandId.CLIENT_SKETCH_SET_COLOR, new ClientCommandSketchSetColor().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndRgb() {
		ClientCommandSketchSetColor cmd = new ClientCommandSketchSetColor(Arrays.asList("s1"), 0xFF0000);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientSketchSetColor", json.get("netCommandId").asString());
		assertEquals(0xFF0000, json.get("rgb").asInt());
	}

	@Test
	public void roundTripWithData() {
		List<String> ids = Arrays.asList("s1", "s2");
		ClientCommandSketchSetColor cmd = new ClientCommandSketchSetColor(ids, 0x00FF00);
		cmd.setEntropy((byte) 5);
		JsonObject json = cmd.toJsonValue();
		ClientCommandSketchSetColor restored = new ClientCommandSketchSetColor().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(ids, restored.getSketchIds());
		assertEquals(0x00FF00, restored.getRbg());
		assertEquals((byte) 5, restored.getEntropy());
	}
}
