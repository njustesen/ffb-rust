package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.marking.SortMode;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_update_player_markings.rs tests.
 */
public class ClientCommandUpdatePlayerMarkingsTest {

	@Test
	public void autoFlag() {
		ClientCommandUpdatePlayerMarkings cmd = new ClientCommandUpdatePlayerMarkings(true, null);
		assertTrue(cmd.isAuto());
	}

	@Test
	public void defaultFalse() {
		assertFalse(new ClientCommandUpdatePlayerMarkings().isAuto());
	}

	@Test
	public void getIdIsClientUpdatePlayerMarkings() {
		assertEquals(NetCommandId.CLIENT_UPDATE_PLAYER_MARKINGS, new ClientCommandUpdatePlayerMarkings().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndUseAutoMarkings() {
		ClientCommandUpdatePlayerMarkings cmd = new ClientCommandUpdatePlayerMarkings(true, SortMode.NONE);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientUpdatePlayerMarkings", json.get("netCommandId").asString());
		assertTrue(json.get("useAutoMarkings").asBoolean());
		assertEquals("NONE", json.get("sortMode").asString());
	}

	@Test
	public void roundTripWithSortModeAndEntropy() {
		ClientCommandUpdatePlayerMarkings cmd = new ClientCommandUpdatePlayerMarkings(true, SortMode.DEFAULT);
		cmd.setEntropy((byte) 14);
		JsonObject json = cmd.toJsonValue();
		ClientCommandUpdatePlayerMarkings restored = new ClientCommandUpdatePlayerMarkings().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 14, restored.getEntropy());
		assertTrue(restored.isAuto());
		assertEquals(SortMode.DEFAULT, restored.getSortMode());
	}

	@Test
	public void roundTripWithNoSortMode() {
		ClientCommandUpdatePlayerMarkings cmd = new ClientCommandUpdatePlayerMarkings();
		JsonObject json = cmd.toJsonValue();
		assertNull(json.get("sortMode"));
		ClientCommandUpdatePlayerMarkings restored = new ClientCommandUpdatePlayerMarkings().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getSortMode());
		assertFalse(restored.isAuto());
	}
}
