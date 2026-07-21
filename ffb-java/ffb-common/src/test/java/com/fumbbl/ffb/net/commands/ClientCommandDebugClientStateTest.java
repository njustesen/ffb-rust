package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_debug_client_state.rs tests.
 */
public class ClientCommandDebugClientStateTest {

	@Test
	public void defaultHasNoStateId() {
		ClientCommandDebugClientState cmd = new ClientCommandDebugClientState();
		assertNull(cmd.getClientStateId());
	}

	@Test
	public void storesClientStateId() {
		ClientCommandDebugClientState cmd = new ClientCommandDebugClientState(ClientStateId.LOGIN);
		assertEquals(ClientStateId.LOGIN, cmd.getClientStateId());
	}

	@Test
	public void getIdIsClientDebugClientState() {
		assertEquals(NetCommandId.CLIENT_DEBUG_CLIENT_STATE, new ClientCommandDebugClientState().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndClientStateId() {
		ClientCommandDebugClientState cmd = new ClientCommandDebugClientState(ClientStateId.BLOCK);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientDebugClientState", json.get("netCommandId").asString());
		assertEquals("block", json.get("clientStateId").asString());
	}

	@Test
	public void roundTripWithStateIdAndEntropy() {
		ClientCommandDebugClientState cmd = new ClientCommandDebugClientState(ClientStateId.KICKOFF);
		cmd.setEntropy((byte) 5);
		JsonObject json = cmd.toJsonValue();
		ClientCommandDebugClientState restored = new ClientCommandDebugClientState().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 5, restored.getEntropy());
		assertEquals(ClientStateId.KICKOFF, restored.getClientStateId());
	}

	@Test
	public void roundTripWithNoStateId() {
		ClientCommandDebugClientState cmd = new ClientCommandDebugClientState();
		JsonObject json = cmd.toJsonValue();
		ClientCommandDebugClientState restored = new ClientCommandDebugClientState().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getClientStateId());
	}
}
