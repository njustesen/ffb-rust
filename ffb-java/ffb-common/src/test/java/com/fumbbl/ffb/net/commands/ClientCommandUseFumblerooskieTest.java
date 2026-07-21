package com.fumbbl.ffb.net.commands;

import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_use_fumblerooskie.rs tests.
 */
public class ClientCommandUseFumblerooskieTest {

	@Test
	public void getIdIsClientUseFumblerooskie() {
		assertEquals(NetCommandId.CLIENT_USE_FUMBLEROOSKIE, new ClientCommandUseFumblerooskie().getId());
	}

	@Test
	public void toJsonValueHasNetCommandId() {
		JsonObject json = new ClientCommandUseFumblerooskie().toJsonValue();
		assertEquals("clientUseFumblerooskie", json.get("netCommandId").asString());
	}

	@Test
	public void roundTripWithEntropy() {
		ClientCommandUseFumblerooskie cmd = new ClientCommandUseFumblerooskie();
		cmd.setEntropy((byte) 5);
		JsonObject json = cmd.toJsonValue();
		ClientCommandUseFumblerooskie restored = (ClientCommandUseFumblerooskie) new ClientCommandUseFumblerooskie()
			.initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 5, restored.getEntropy());
	}

	@Test
	public void roundTripWithNoEntropy() {
		ClientCommandUseFumblerooskie cmd = new ClientCommandUseFumblerooskie();
		JsonObject json = cmd.toJsonValue();
		ClientCommandUseFumblerooskie restored = (ClientCommandUseFumblerooskie) new ClientCommandUseFumblerooskie()
			.initFrom(NetCommandTestUtil.gameSource(), json);
		assertFalse(restored.hasEntropy());
	}
}
