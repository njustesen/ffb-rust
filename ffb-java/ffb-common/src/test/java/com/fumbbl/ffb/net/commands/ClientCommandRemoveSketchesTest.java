package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_remove_sketches.rs tests.
 *
 * Java differences from the Rust source:
 * - Java has no {@code add_id} mutator (ids are set only via the constructor), so the Rust
 *   {@code add_id_increments_len} test is inexpressible and is skipped.
 * - The default (no-arg) Java command leaves {@code ids} null rather than an empty list, so the
 *   "empty" assertions below are written against null (Java-correct).
 */
public class ClientCommandRemoveSketchesTest {

	@Test
	public void idsStored() {
		ClientCommandRemoveSketches cmd = new ClientCommandRemoveSketches(Arrays.asList("id1", "id2"));
		assertEquals(2, cmd.getIds().size());
		assertEquals("id1", cmd.getIds().get(0));
	}

	@Test
	public void defaultIsEmpty() {
		ClientCommandRemoveSketches cmd = new ClientCommandRemoveSketches();
		assertNull(cmd.getIds());
	}

	@Test
	public void getIdIsClientRemoveSketches() {
		assertEquals(NetCommandId.CLIENT_REMOVE_SKETCHES, new ClientCommandRemoveSketches().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndIds() {
		ClientCommandRemoveSketches cmd = new ClientCommandRemoveSketches(Arrays.asList("sk-1"));
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientRemoveSketches", json.get("netCommandId").asString());
		assertEquals(1, json.get("ids").asArray().size());
		assertEquals("sk-1", json.get("ids").asArray().get(0).asString());
	}

	@Test
	public void roundTripWithData() {
		ClientCommandRemoveSketches cmd = new ClientCommandRemoveSketches(Arrays.asList("a", "b"));
		cmd.setEntropy((byte) 3);
		JsonObject json = cmd.toJsonValue();
		ClientCommandRemoveSketches restored = new ClientCommandRemoveSketches().initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 3, restored.getEntropy());
		assertEquals(Arrays.asList("a", "b"), restored.getIds());
	}

	@Test
	public void roundTripDefault() {
		ClientCommandRemoveSketches cmd = new ClientCommandRemoveSketches();
		JsonObject json = cmd.toJsonValue();
		ClientCommandRemoveSketches restored = new ClientCommandRemoveSketches().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getIds());
		assertFalse(restored.hasEntropy());
		assertNull(json.get("ids"));
	}
}
