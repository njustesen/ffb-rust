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
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_sketch_set_label.rs tests.
 *
 * Note: the label is written under the {@code "text"} wire key. The default command
 * leaves {@code sketchIds} null, and {@code initFrom} calls {@code Arrays.asList(null)}
 * which NPEs, so the Rust {@code round_trip_default} test is inexpressible in Java and
 * is SKIPPED.
 */
public class ClientCommandSketchSetLabelTest {

	@Test
	public void fieldsStored() {
		ClientCommandSketchSetLabel cmd = new ClientCommandSketchSetLabel(Arrays.asList("s1"), "attack");
		assertEquals(1, cmd.getSketchIds().size());
		assertEquals("attack", cmd.getLabel());
	}

	@Test
	public void defaultIsEmpty() {
		// Rust default has an empty vec; Java leaves sketchIds null.
		ClientCommandSketchSetLabel cmd = new ClientCommandSketchSetLabel();
		assertNull(cmd.getSketchIds());
		assertNull(cmd.getLabel());
	}

	@Test
	public void getIdIsClientSketchSetLabel() {
		assertEquals(NetCommandId.CLIENT_SKETCH_SET_LABEL, new ClientCommandSketchSetLabel().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndTextKey() {
		ClientCommandSketchSetLabel cmd = new ClientCommandSketchSetLabel(Arrays.asList("s1"), "attack");
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientSketchSetLabel", json.get("netCommandId").asString());
		assertEquals("attack", json.get("text").asString());
	}

	@Test
	public void roundTripWithData() {
		List<String> ids = Arrays.asList("s1", "s2");
		ClientCommandSketchSetLabel cmd = new ClientCommandSketchSetLabel(ids, "defend");
		cmd.setEntropy((byte) 6);
		JsonObject json = cmd.toJsonValue();
		ClientCommandSketchSetLabel restored = new ClientCommandSketchSetLabel().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(ids, restored.getSketchIds());
		assertEquals("defend", restored.getLabel());
		assertEquals((byte) 6, restored.getEntropy());
	}
}
