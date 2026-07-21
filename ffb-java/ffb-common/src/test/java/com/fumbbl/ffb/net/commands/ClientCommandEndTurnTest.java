package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.TurnMode;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import java.util.HashMap;
import java.util.Map;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_end_turn.rs tests.
 */
public class ClientCommandEndTurnTest {

	@Test
	public void defaultIsEmpty() {
		ClientCommandEndTurn cmd = new ClientCommandEndTurn();
		assertNull(cmd.getTurnMode());
		assertTrue(cmd.getPlayerCoordinates().isEmpty());
	}

	@Test
	public void turnModeStored() {
		ClientCommandEndTurn cmd = new ClientCommandEndTurn(TurnMode.REGULAR, null);
		assertEquals(TurnMode.REGULAR, cmd.getTurnMode());
	}

	@Test
	public void playerCoordinatesStored() {
		Map<String, FieldCoordinate> coords = new HashMap<>();
		coords.put("p1", new FieldCoordinate(5, 5));
		ClientCommandEndTurn cmd = new ClientCommandEndTurn(null, coords);
		assertEquals(1, cmd.getPlayerCoordinates().size());
	}

	@Test
	public void getIdIsClientEndTurn() {
		assertEquals(NetCommandId.CLIENT_END_TURN, new ClientCommandEndTurn().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndTurnMode() {
		ClientCommandEndTurn cmd = new ClientCommandEndTurn(TurnMode.BLITZ, null);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientEndTurn", json.get("netCommandId").asString());
		assertEquals("blitz", json.get("turnMode").asString());
	}

	@Test
	public void roundTripWithCoordinatesAndEntropy() {
		Map<String, FieldCoordinate> coords = new HashMap<>();
		coords.put("p1", new FieldCoordinate(3, 4));
		ClientCommandEndTurn cmd = new ClientCommandEndTurn(TurnMode.REGULAR, coords);
		cmd.setEntropy((byte) 9);
		JsonObject json = cmd.toJsonValue();
		ClientCommandEndTurn restored = new ClientCommandEndTurn().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 9, restored.getEntropy());
		assertEquals(TurnMode.REGULAR, restored.getTurnMode());
		assertEquals(new FieldCoordinate(3, 4), restored.getPlayerCoordinates().get("p1"));
	}

	@Test
	public void roundTripWithNoTurnModeAndEmptyCoordinates() {
		ClientCommandEndTurn cmd = new ClientCommandEndTurn();
		JsonObject json = cmd.toJsonValue();
		ClientCommandEndTurn restored = new ClientCommandEndTurn().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getTurnMode());
		assertTrue(restored.getPlayerCoordinates().isEmpty());
	}
}
