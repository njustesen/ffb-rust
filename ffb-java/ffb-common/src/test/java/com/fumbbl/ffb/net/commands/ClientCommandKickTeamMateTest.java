package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_kick_team_mate.rs tests.
 */
public class ClientCommandKickTeamMateTest {

	@Test
	public void defaultNumDiceIsZero() {
		ClientCommandKickTeamMate cmd = new ClientCommandKickTeamMate();
		assertEquals(0, cmd.getNumDice());
	}

	@Test
	public void storesPlayerIdsAndNumDice() {
		ClientCommandKickTeamMate cmd = new ClientCommandKickTeamMate("acting_1", "kicked_1", 2);
		assertEquals("kicked_1", cmd.getKickedPlayerId());
		assertEquals("acting_1", cmd.getActingPlayerId());
		assertEquals(2, cmd.getNumDice());
	}

	@Test
	public void defaultIdsAreNone() {
		ClientCommandKickTeamMate cmd = new ClientCommandKickTeamMate();
		assertNull(cmd.getKickedPlayerId());
		assertNull(cmd.getActingPlayerId());
	}

	@Test
	public void getIdIsClientKickTeamMate() {
		assertEquals(NetCommandId.CLIENT_KICK_TEAM_MATE, new ClientCommandKickTeamMate().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndNrOfDice() {
		ClientCommandKickTeamMate cmd = new ClientCommandKickTeamMate("acting_1", "kicked_1", 2);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientKickTeamMate", json.get("netCommandId").asString());
		assertEquals(2, json.get("nrOfDice").asInt());
	}

	@Test
	public void roundTripWithFieldsAndEntropy() {
		ClientCommandKickTeamMate cmd = new ClientCommandKickTeamMate("acting_1", "kicked_1", 3);
		cmd.setEntropy((byte) 7);
		JsonObject json = cmd.toJsonValue();
		ClientCommandKickTeamMate restored = new ClientCommandKickTeamMate().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 7, restored.getEntropy());
		assertEquals("kicked_1", restored.getKickedPlayerId());
		assertEquals("acting_1", restored.getActingPlayerId());
		assertEquals(3, restored.getNumDice());
	}

	@Test
	public void roundTripDefault() {
		ClientCommandKickTeamMate cmd = new ClientCommandKickTeamMate();
		JsonObject json = cmd.toJsonValue();
		ClientCommandKickTeamMate restored = new ClientCommandKickTeamMate().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getKickedPlayerId());
		assertNull(restored.getActingPlayerId());
		assertEquals(0, restored.getNumDice());
	}
}
