package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.ClientMode;
import com.fumbbl.ffb.FactoryType.FactoryContext;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/server_command_join.rs tests.
 */
public class ServerCommandJoinTest {

	@Test
	public void fieldsStored() {
		ServerCommandJoin cmd = new ServerCommandJoin("Alice", ClientMode.PLAYER, new String[] { "Alice" },
			Arrays.asList("Bob"), "");
		assertEquals("Alice", cmd.getCoach());
		assertEquals(ClientMode.PLAYER, cmd.getClientMode());
		assertEquals(1, cmd.getSpectatorCount());
	}

	@Test
	public void defaultEmpty() {
		ServerCommandJoin cmd = new ServerCommandJoin();
		assertNull(cmd.getCoach());
		assertEquals(0, cmd.getPlayerNames().length);
	}

	@Test
	public void getIdIsServerJoin() {
		assertEquals(NetCommandId.SERVER_JOIN, new ServerCommandJoin().getId());
	}

	@Test
	public void getContextIsApplication() {
		assertEquals(FactoryContext.APPLICATION, new ServerCommandJoin().getContext());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndCoach() {
		ServerCommandJoin cmd = new ServerCommandJoin("Alice", ClientMode.SPECTATOR, new String[] {},
			Arrays.asList("Bob"), "");
		JsonObject json = cmd.toJsonValue();
		assertEquals("serverJoin", json.get("netCommandId").asString());
		assertEquals("Alice", json.get("coach").asString());
		assertEquals("spectator", json.get("clientMode").asString());
		assertEquals(1, json.get("spectators").asInt());
	}

	@Test
	public void roundTripWithPlayersAndSpectators() {
		ServerCommandJoin cmd = new ServerCommandJoin("Alice", ClientMode.PLAYER, new String[] { "Alice", "Bob" },
			Arrays.asList("Carol"), "replay1");
		cmd.setCommandNr(6);
		JsonObject json = cmd.toJsonValue();
		ServerCommandJoin restored = new ServerCommandJoin().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(6, restored.getCommandNr());
		assertEquals("Alice", restored.getCoach());
		assertEquals(ClientMode.PLAYER, restored.getClientMode());
		assertArrayEquals(new String[] { "Alice", "Bob" }, restored.getPlayerNames());
		assertEquals(Arrays.asList("Carol"), restored.getSpectators());
		assertEquals("replay1", restored.getReplayName());
	}

	@Test
	public void roundTripWithDefaults() {
		ServerCommandJoin cmd = new ServerCommandJoin();
		JsonObject json = cmd.toJsonValue();
		ServerCommandJoin restored = new ServerCommandJoin().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getCoach());
		assertEquals(0, restored.getPlayerNames().length);
		assertTrue(restored.getSpectators().isEmpty());
	}
}
