package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_illegal_procedure.rs tests.
 */
public class ClientCommandIllegalProcedureTest {

	@Test
	public void getIdIsClientIllegalProcedure() {
		assertEquals(NetCommandId.CLIENT_ILLEGAL_PROCEDURE, new ClientCommandIllegalProcedure().getId());
	}

	@Test
	public void toJsonValueHasNetCommandId() {
		ClientCommandIllegalProcedure cmd = new ClientCommandIllegalProcedure();
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientIllegalProcedure", json.get("netCommandId").asString());
	}

	@Test
	public void roundTripWithEntropy() {
		ClientCommandIllegalProcedure cmd = new ClientCommandIllegalProcedure();
		cmd.setEntropy((byte) 1);
		JsonObject json = cmd.toJsonValue();
		ClientCommandIllegalProcedure restored = new ClientCommandIllegalProcedure().initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 1, restored.getEntropy());
	}

	@Test
	public void roundTripDefault() {
		ClientCommandIllegalProcedure cmd = new ClientCommandIllegalProcedure();
		JsonObject json = cmd.toJsonValue();
		ClientCommandIllegalProcedure restored = new ClientCommandIllegalProcedure().initFrom(NetCommandTestUtil.gameSource(), json);
		assertFalse(restored.hasEntropy());
	}
}
