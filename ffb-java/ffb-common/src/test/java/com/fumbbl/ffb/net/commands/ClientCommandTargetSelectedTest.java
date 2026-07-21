package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_target_selected.rs tests.
 */
public class ClientCommandTargetSelectedTest {

	@Test
	public void defaultHasNoTarget() {
		ClientCommandTargetSelected cmd = new ClientCommandTargetSelected();
		assertNull(cmd.getTargetPlayerId());
	}

	@Test
	public void withTargetStoresValue() {
		ClientCommandTargetSelected cmd = new ClientCommandTargetSelected("player-123");
		assertEquals("player-123", cmd.getTargetPlayerId());
	}

	@Test
	public void getIdIsClientTargetSelected() {
		assertEquals(NetCommandId.CLIENT_TARGET_SELECTED, new ClientCommandTargetSelected().getId());
	}

	@Test
	public void toJsonValueUsesPlayerIdWireKey() {
		ClientCommandTargetSelected cmd = new ClientCommandTargetSelected("player-123");
		JsonObject json = cmd.toJsonValue();
		assertEquals("targetSelected", json.get("netCommandId").asString());
		assertEquals("player-123", json.get("playerId").asString());
	}

	@Test
	public void roundTripWithTargetAndEntropy() {
		ClientCommandTargetSelected cmd = new ClientCommandTargetSelected("player-123");
		cmd.setEntropy((byte) 2);
		JsonObject json = cmd.toJsonValue();
		ClientCommandTargetSelected restored = (ClientCommandTargetSelected)
			new ClientCommandTargetSelected().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 2, restored.getEntropy());
		assertEquals("player-123", restored.getTargetPlayerId());
	}

	@Test
	public void roundTripWithNoTarget() {
		ClientCommandTargetSelected cmd = new ClientCommandTargetSelected();
		JsonObject json = cmd.toJsonValue();
		ClientCommandTargetSelected restored = (ClientCommandTargetSelected)
			new ClientCommandTargetSelected().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getTargetPlayerId());
	}
}
