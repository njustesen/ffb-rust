package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_move.rs tests.
 */
public class ClientCommandMoveTest {

	@Test
	public void fieldsStored() {
		FieldCoordinate from = new FieldCoordinate(3, 4);
		FieldCoordinate[] to = { new FieldCoordinate(4, 4), new FieldCoordinate(5, 4) };
		ClientCommandMove cmd = new ClientCommandMove("p1", from, to, null);
		assertEquals("p1", cmd.getActingPlayerId());
		assertEquals(from, cmd.getCoordinateFrom());
		assertEquals(2, cmd.getCoordinatesTo().length);
		assertNull(cmd.getBallAndChainRrSetting());
	}

	@Test
	public void ballAndChainSetting() {
		ClientCommandMove cmd = new ClientCommandMove("p1", new FieldCoordinate(1, 1), new FieldCoordinate[0], "ALWAYS");
		assertEquals("ALWAYS", cmd.getBallAndChainRrSetting());
	}

	@Test
	public void defaultEmpty() {
		ClientCommandMove cmd = new ClientCommandMove();
		assertNull(cmd.getActingPlayerId());
		assertEquals(0, cmd.getCoordinatesTo().length);
	}

	@Test
	public void getIdIsClientMove() {
		assertEquals(NetCommandId.CLIENT_MOVE, new ClientCommandMove().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndCoordinates() {
		FieldCoordinate from = new FieldCoordinate(2, 2);
		FieldCoordinate[] to = { new FieldCoordinate(3, 2) };
		ClientCommandMove cmd = new ClientCommandMove("p1", from, to, "ALWAYS");
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientMove", json.get("netCommandId").asString());
		assertEquals(2, json.get("coordinateFrom").asArray().get(0).asInt());
		assertEquals(2, json.get("coordinateFrom").asArray().get(1).asInt());
		assertEquals(1, json.get("coordinatesTo").asArray().size());
		assertEquals(3, json.get("coordinatesTo").asArray().get(0).asArray().get(0).asInt());
		assertEquals(2, json.get("coordinatesTo").asArray().get(0).asArray().get(1).asInt());
		assertEquals("ALWAYS", json.get("ballAndChainReRollSetting").asString());
	}

	@Test
	public void roundTripWithData() {
		FieldCoordinate from = new FieldCoordinate(6, 7);
		FieldCoordinate[] to = { new FieldCoordinate(7, 7), new FieldCoordinate(8, 7) };
		ClientCommandMove cmd = new ClientCommandMove("p9", from, to, "NEVER");
		cmd.setEntropy((byte) 4);
		JsonObject json = cmd.toJsonValue();
		ClientCommandMove restored = new ClientCommandMove().initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 4, restored.getEntropy());
		assertEquals("p9", restored.getActingPlayerId());
		assertEquals(from, restored.getCoordinateFrom());
		assertEquals(2, restored.getCoordinatesTo().length);
		assertEquals(new FieldCoordinate(7, 7), restored.getCoordinatesTo()[0]);
		assertEquals(new FieldCoordinate(8, 7), restored.getCoordinatesTo()[1]);
		assertEquals("NEVER", restored.getBallAndChainRrSetting());
	}

	@Test
	public void roundTripDefault() {
		ClientCommandMove cmd = new ClientCommandMove();
		JsonObject json = cmd.toJsonValue();
		ClientCommandMove restored = new ClientCommandMove().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getActingPlayerId());
		assertNull(restored.getCoordinateFrom());
		assertEquals(0, restored.getCoordinatesTo().length);
		assertNull(restored.getBallAndChainRrSetting());
		assertFalse(restored.hasEntropy());
	}
}
