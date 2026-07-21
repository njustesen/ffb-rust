package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_clear_sketches.rs tests.
 */
public class ClientCommandClearSketchesTest {

	@Test
	public void requiresControlIsTrue() {
		assertTrue(new ClientCommandClearSketches().requiresControl());
	}

	@Test
	public void getIdIsClientClearSketches() {
		assertEquals(NetCommandId.CLIENT_CLEAR_SKETCHES, new ClientCommandClearSketches().getId());
	}

	@Test
	public void toJsonValueHasNetCommandId() {
		JsonObject json = new ClientCommandClearSketches().toJsonValue();
		assertEquals("clientClearSketches", json.get("netCommandId").asString());
	}

	@Test
	public void roundTripWithEntropy() {
		ClientCommandClearSketches cmd = new ClientCommandClearSketches();
		cmd.setEntropy((byte) 2);
		JsonObject json = cmd.toJsonValue();
		ClientCommandClearSketches restored = new ClientCommandClearSketches().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 2, restored.getEntropy());
	}

	@Test
	public void roundTripWithNoEntropy() {
		ClientCommandClearSketches cmd = new ClientCommandClearSketches();
		JsonObject json = cmd.toJsonValue();
		ClientCommandClearSketches restored = new ClientCommandClearSketches().initFrom(NetCommandTestUtil.gameSource(), json);
		assertFalse(restored.hasEntropy());
	}
}
