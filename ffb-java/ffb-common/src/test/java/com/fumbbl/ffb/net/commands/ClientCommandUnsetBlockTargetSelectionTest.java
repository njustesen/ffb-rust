package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_unset_block_target_selection.rs tests.
 */
public class ClientCommandUnsetBlockTargetSelectionTest {

	@Test
	public void defaultHasNoPlayerId() {
		ClientCommandUnsetBlockTargetSelection cmd = new ClientCommandUnsetBlockTargetSelection();
		assertNull(cmd.getPlayerId());
	}

	@Test
	public void withPlayerIdStoresValue() {
		ClientCommandUnsetBlockTargetSelection cmd = new ClientCommandUnsetBlockTargetSelection("p-99");
		assertEquals("p-99", cmd.getPlayerId());
	}

	@Test
	public void getIdIsClientUnsetBlockTargetSelection() {
		assertEquals(NetCommandId.CLIENT_UNSET_BLOCK_TARGET_SELECTION, new ClientCommandUnsetBlockTargetSelection().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndPlayerId() {
		ClientCommandUnsetBlockTargetSelection cmd = new ClientCommandUnsetBlockTargetSelection("p-1");
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientUnsetBlockTargetSelection", json.get("netCommandId").asString());
		assertEquals("p-1", json.get("playerId").asString());
	}

	@Test
	public void roundTripWithPlayerIdAndEntropy() {
		ClientCommandUnsetBlockTargetSelection cmd = new ClientCommandUnsetBlockTargetSelection("p-2");
		cmd.setEntropy((byte) 2);
		JsonObject json = cmd.toJsonValue();
		ClientCommandUnsetBlockTargetSelection restored = (ClientCommandUnsetBlockTargetSelection)
			new ClientCommandUnsetBlockTargetSelection().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 2, restored.getEntropy());
		assertEquals("p-2", restored.getPlayerId());
	}

	@Test
	public void roundTripWithNoPlayerId() {
		ClientCommandUnsetBlockTargetSelection cmd = new ClientCommandUnsetBlockTargetSelection();
		JsonObject json = cmd.toJsonValue();
		ClientCommandUnsetBlockTargetSelection restored = (ClientCommandUnsetBlockTargetSelection)
			new ClientCommandUnsetBlockTargetSelection().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getPlayerId());
	}
}
