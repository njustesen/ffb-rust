package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_confirm.rs tests.
 */
public class ClientCommandConfirmTest {

	@Test
	public void getIdIsClientConfirm() {
		assertEquals(NetCommandId.CLIENT_CONFIRM, new ClientCommandConfirm().getId());
	}

	@Test
	public void toJsonValueHasNetCommandId() {
		JsonObject json = new ClientCommandConfirm().toJsonValue();
		assertEquals("clientConfirm", json.get("netCommandId").asString());
	}

	@Test
	public void roundTripWithEntropy() {
		ClientCommandConfirm cmd = new ClientCommandConfirm();
		cmd.setEntropy((byte) 11);
		JsonObject json = cmd.toJsonValue();
		ClientCommandConfirm restored = new ClientCommandConfirm().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 11, restored.getEntropy());
	}

	@Test
	public void roundTripWithNoEntropy() {
		ClientCommandConfirm cmd = new ClientCommandConfirm();
		JsonObject json = cmd.toJsonValue();
		ClientCommandConfirm restored = new ClientCommandConfirm().initFrom(NetCommandTestUtil.gameSource(), json);
		assertFalse(restored.hasEntropy());
	}
}
