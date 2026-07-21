package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.ClientMode;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_join.rs tests.
 */
public class ClientCommandJoinTest {

	@Test
	public void defaultGameIdIsZero() {
		ClientCommandJoin cmd = new ClientCommandJoin();
		assertEquals(0L, cmd.getGameId());
	}

	@Test
	public void storesCoachAndGameId() {
		ClientCommandJoin cmd = new ClientCommandJoin();
		cmd.setCoach("TestCoach");
		cmd.setGameId(42);
		assertEquals("TestCoach", cmd.getCoach());
		assertEquals(42L, cmd.getGameId());
	}

	@Test
	public void teamIdStored() {
		ClientCommandJoin cmd = new ClientCommandJoin();
		cmd.setTeamId("team-abc");
		assertEquals("team-abc", cmd.getTeamId());
	}

	@Test
	public void getIdIsClientJoin() {
		assertEquals(NetCommandId.CLIENT_JOIN, new ClientCommandJoin().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndClientMode() {
		ClientCommandJoin cmd = new ClientCommandJoin(ClientMode.PLAYER);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientJoin", json.get("netCommandId").asString());
		assertEquals("player", json.get("clientMode").asString());
	}

	@Test
	public void roundTripWithAllFieldsAndEntropy() {
		ClientCommandJoin cmd = new ClientCommandJoin(ClientMode.SPECTATOR);
		cmd.setEntropy((byte) 1);
		cmd.setCoach("Coach");
		cmd.setPassword("pw");
		cmd.setGameName("Game1");
		cmd.setTeamId("t1");
		cmd.setTeamName("Team1");
		cmd.setGameId(77);
		JsonObject json = cmd.toJsonValue();
		ClientCommandJoin restored = new ClientCommandJoin().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 1, restored.getEntropy());
		assertEquals("Coach", restored.getCoach());
		assertEquals("pw", restored.getPassword());
		assertEquals("Game1", restored.getGameName());
		assertEquals("t1", restored.getTeamId());
		assertEquals("Team1", restored.getTeamName());
		assertEquals(77L, restored.getGameId());
		assertEquals(ClientMode.SPECTATOR, restored.getClientMode());
	}

	@Test
	public void roundTripDefault() {
		ClientCommandJoin cmd = new ClientCommandJoin();
		JsonObject json = cmd.toJsonValue();
		ClientCommandJoin restored = new ClientCommandJoin().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getCoach());
		assertNull(restored.getClientMode());
		assertEquals(0L, restored.getGameId());
	}
}
