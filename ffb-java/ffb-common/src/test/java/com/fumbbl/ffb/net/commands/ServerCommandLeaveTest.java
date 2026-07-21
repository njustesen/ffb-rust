package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.ClientMode;
import com.fumbbl.ffb.FactoryType.FactoryContext;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/server_command_leave.rs tests.
 */
public class ServerCommandLeaveTest {

	@Test
	public void fieldsStored() {
		ServerCommandLeave cmd = new ServerCommandLeave("Bob", ClientMode.SPECTATOR, Arrays.asList("Charlie"));
		assertEquals("Bob", cmd.getCoach());
		assertEquals(ClientMode.SPECTATOR, cmd.getClientMode());
		assertEquals(1, cmd.getSpectatorCount());
	}

	@Test
	public void defaultEmpty() {
		ServerCommandLeave cmd = new ServerCommandLeave();
		assertNull(cmd.getCoach());
	}

	@Test
	public void getIdIsServerLeave() {
		assertEquals(NetCommandId.SERVER_LEAVE, new ServerCommandLeave().getId());
	}

	@Test
	public void getContextIsApplication() {
		assertEquals(FactoryContext.APPLICATION, new ServerCommandLeave().getContext());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndCoach() {
		ServerCommandLeave cmd = new ServerCommandLeave("Bob", ClientMode.SPECTATOR, Arrays.asList("Charlie"));
		JsonObject json = cmd.toJsonValue();
		assertEquals("serverLeave", json.get("netCommandId").asString());
		assertEquals("Bob", json.get("coach").asString());
		assertEquals("spectator", json.get("clientMode").asString());
		assertEquals(1, json.get("spectators").asInt());
	}

	@Test
	public void roundTripWithSpectators() {
		ServerCommandLeave cmd = new ServerCommandLeave("Bob", ClientMode.PLAYER, Arrays.asList("Charlie", "Dave"));
		cmd.setCommandNr(4);
		JsonObject json = cmd.toJsonValue();
		ServerCommandLeave restored = new ServerCommandLeave().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(4, restored.getCommandNr());
		assertEquals("Bob", restored.getCoach());
		assertEquals(ClientMode.PLAYER, restored.getClientMode());
		assertEquals(Arrays.asList("Charlie", "Dave"), restored.getSpectators());
	}

	@Test
	public void roundTripWithDefaults() {
		ServerCommandLeave cmd = new ServerCommandLeave();
		JsonObject json = cmd.toJsonValue();
		ServerCommandLeave restored = new ServerCommandLeave().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getCoach());
		assertTrue(restored.getSpectators().isEmpty());
	}
}
