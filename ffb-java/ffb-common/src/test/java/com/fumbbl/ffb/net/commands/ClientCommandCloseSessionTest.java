package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_close_session.rs tests.
 */
public class ClientCommandCloseSessionTest {

	@Test
	public void getIdIsClientCloseSession() {
		assertEquals(NetCommandId.CLIENT_CLOSE_SESSION, new ClientCommandCloseSession().getId());
	}

	@Test
	public void toJsonValueHasNetCommandId() {
		JsonObject json = new ClientCommandCloseSession().toJsonValue();
		assertEquals("clientCloseSession", json.get("netCommandId").asString());
	}

	@Test
	public void roundTripWithEntropy() {
		ClientCommandCloseSession cmd = new ClientCommandCloseSession();
		cmd.setEntropy((byte) 5);
		JsonObject json = cmd.toJsonValue();
		ClientCommandCloseSession restored = new ClientCommandCloseSession().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 5, restored.getEntropy());
	}

	@Test
	public void roundTripWithNoEntropy() {
		ClientCommandCloseSession cmd = new ClientCommandCloseSession();
		JsonObject json = cmd.toJsonValue();
		ClientCommandCloseSession restored = new ClientCommandCloseSession().initFrom(NetCommandTestUtil.gameSource(), json);
		assertFalse(restored.hasEntropy());
	}
}
