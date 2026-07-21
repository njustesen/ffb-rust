package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.PlayerChoiceMode;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_player_choice.rs tests.
 * Java has no fluent {@code with_mode}; the mode is supplied via the
 * {@code (PlayerChoiceMode, Player[])} constructor with an empty player array.
 */
public class ClientCommandPlayerChoiceTest {

	@Test
	public void modeStoredAndIdsAdded() {
		ClientCommandPlayerChoice cmd = new ClientCommandPlayerChoice(PlayerChoiceMode.BLOCK, new Player[0]);
		cmd.addPlayerId("p1");
		cmd.addPlayerId("p2");
		assertEquals(PlayerChoiceMode.BLOCK, cmd.getPlayerChoiceMode());
		assertEquals(2, cmd.getPlayerIds().length);
	}

	@Test
	public void defaultIsEmpty() {
		ClientCommandPlayerChoice cmd = new ClientCommandPlayerChoice();
		assertNull(cmd.getPlayerChoiceMode());
		assertEquals(0, cmd.getPlayerIds().length);
	}

	@Test
	public void addSingleIdLenIsOne() {
		ClientCommandPlayerChoice cmd = new ClientCommandPlayerChoice();
		cmd.addPlayerId("p99");
		assertEquals(1, cmd.getPlayerIds().length);
	}

	@Test
	public void getIdIsClientPlayerChoice() {
		assertEquals(NetCommandId.CLIENT_PLAYER_CHOICE, new ClientCommandPlayerChoice().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndMode() {
		ClientCommandPlayerChoice cmd = new ClientCommandPlayerChoice(PlayerChoiceMode.CARD, new Player[0]);
		cmd.addPlayerId("p1");
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientPlayerChoice", json.get("netCommandId").asString());
		assertEquals("card", json.get("playerChoiceMode").asString());
		assertEquals(1, json.get("playerIds").asArray().size());
		assertEquals("p1", json.get("playerIds").asArray().get(0).asString());
	}

	@Test
	public void roundTripWithData() {
		ClientCommandPlayerChoice cmd = new ClientCommandPlayerChoice(PlayerChoiceMode.MVP, new Player[0]);
		cmd.addPlayerId("p1");
		cmd.addPlayerId("p2");
		cmd.setEntropy((byte) 8);
		JsonObject json = cmd.toJsonValue();
		ClientCommandPlayerChoice restored =
			new ClientCommandPlayerChoice().initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 8, restored.getEntropy());
		assertEquals(PlayerChoiceMode.MVP, restored.getPlayerChoiceMode());
		assertEquals(2, restored.getPlayerIds().length);
		assertEquals("p1", restored.getPlayerIds()[0]);
		assertEquals("p2", restored.getPlayerIds()[1]);
	}

	@Test
	public void roundTripDefault() {
		ClientCommandPlayerChoice cmd = new ClientCommandPlayerChoice();
		JsonObject json = cmd.toJsonValue();
		ClientCommandPlayerChoice restored =
			new ClientCommandPlayerChoice().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getPlayerChoiceMode());
		assertEquals(0, restored.getPlayerIds().length);
		assertFalse(restored.hasEntropy());
	}
}
