package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_request_version.rs tests.
 */
public class ClientCommandRequestVersionTest {

	@Test
	public void getIdIsClientRequestVersion() {
		assertEquals(NetCommandId.CLIENT_REQUEST_VERSION, new ClientCommandRequestVersion().getId());
	}

	@Test
	public void toJsonValueHasNetCommandId() {
		ClientCommandRequestVersion cmd = new ClientCommandRequestVersion();
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientRequestVersion", json.get("netCommandId").asString());
	}

	@Test
	public void roundTripWithEntropy() {
		ClientCommandRequestVersion cmd = new ClientCommandRequestVersion();
		cmd.setEntropy((byte) 5);
		JsonObject json = cmd.toJsonValue();
		ClientCommandRequestVersion restored = new ClientCommandRequestVersion().initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 5, restored.getEntropy());
	}

	@Test
	public void roundTripDefault() {
		ClientCommandRequestVersion cmd = new ClientCommandRequestVersion();
		JsonObject json = cmd.toJsonValue();
		ClientCommandRequestVersion restored = new ClientCommandRequestVersion().initFrom(NetCommandTestUtil.gameSource(), json);
		assertFalse(restored.hasEntropy());
	}
}
