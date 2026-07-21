package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_blitz_move.rs tests.
 */
public class ClientCommandBlitzMoveTest {

	@Test
	public void fieldsStoredCorrectly() {
		FieldCoordinate from = new FieldCoordinate(1, 1);
		FieldCoordinate[] to = new FieldCoordinate[] { new FieldCoordinate(2, 2), new FieldCoordinate(3, 3) };
		ClientCommandBlitzMove cmd = new ClientCommandBlitzMove("p1", from, to);
		assertEquals("p1", cmd.getActingPlayerId());
		assertEquals(from, cmd.getCoordinateFrom());
		assertEquals(2, cmd.getCoordinatesTo().length);
	}

	@Test
	public void defaultAllNone() {
		ClientCommandBlitzMove cmd = new ClientCommandBlitzMove();
		assertNull(cmd.getActingPlayerId());
		assertNull(cmd.getCoordinateFrom());
		assertEquals(0, cmd.getCoordinatesTo().length);
	}

	@Test
	public void coordinatesToSliceMatchesInput() {
		FieldCoordinate from = new FieldCoordinate(0, 0);
		FieldCoordinate[] to = new FieldCoordinate[] { new FieldCoordinate(1, 0) };
		ClientCommandBlitzMove cmd = new ClientCommandBlitzMove("p2", from, to);
		assertArrayEquals(to, cmd.getCoordinatesTo());
	}

	@Test
	public void getIdIsClientBlitzMove() {
		assertEquals(NetCommandId.CLIENT_BLITZ_MOVE, new ClientCommandBlitzMove().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndActingPlayerId() {
		ClientCommandBlitzMove cmd = new ClientCommandBlitzMove("p1", new FieldCoordinate(1, 1), new FieldCoordinate[0]);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientBlitzMove", json.get("netCommandId").asString());
		assertEquals("p1", json.get("actingPlayerId").asString());
	}

	@Test
	public void roundTripWithPopulatedData() {
		FieldCoordinate[] to = new FieldCoordinate[] { new FieldCoordinate(2, 2), new FieldCoordinate(3, 3) };
		ClientCommandBlitzMove cmd = new ClientCommandBlitzMove("p1", new FieldCoordinate(1, 1), to);
		cmd.setEntropy((byte) 7);
		JsonObject json = cmd.toJsonValue();
		ClientCommandBlitzMove restored = new ClientCommandBlitzMove().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 7, restored.getEntropy());
		assertEquals("p1", restored.getActingPlayerId());
		assertEquals(new FieldCoordinate(1, 1), restored.getCoordinateFrom());
		assertArrayEquals(to, restored.getCoordinatesTo());
	}

	@Test
	public void roundTripWithDefaultData() {
		ClientCommandBlitzMove cmd = new ClientCommandBlitzMove();
		JsonObject json = cmd.toJsonValue();
		ClientCommandBlitzMove restored = new ClientCommandBlitzMove().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getActingPlayerId());
		assertNull(restored.getCoordinateFrom());
		assertEquals(0, restored.getCoordinatesTo().length);
	}
}
