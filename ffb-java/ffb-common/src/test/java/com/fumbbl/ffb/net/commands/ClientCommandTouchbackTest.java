package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_touchback.rs tests.
 */
public class ClientCommandTouchbackTest {

	@Test
	public void coordinateStored() {
		FieldCoordinate coord = new FieldCoordinate(12, 8);
		ClientCommandTouchback cmd = new ClientCommandTouchback(coord);
		assertEquals(coord, cmd.getBallCoordinate());
	}

	@Test
	public void defaultIsEmpty() {
		ClientCommandTouchback cmd = new ClientCommandTouchback();
		assertNull(cmd.getBallCoordinate());
	}

	@Test
	public void getIdIsClientTouchback() {
		assertEquals(NetCommandId.CLIENT_TOUCHBACK, new ClientCommandTouchback().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndBallCoordinate() {
		ClientCommandTouchback cmd = new ClientCommandTouchback(new FieldCoordinate(3, 4));
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientTouchback", json.get("netCommandId").asString());
		assertEquals(3, json.get("ballCoordinate").asArray().get(0).asInt());
		assertEquals(4, json.get("ballCoordinate").asArray().get(1).asInt());
	}

	@Test
	public void roundTripWithCoordinateAndEntropy() {
		ClientCommandTouchback cmd = new ClientCommandTouchback(new FieldCoordinate(5, 6));
		cmd.setEntropy((byte) 8);
		JsonObject json = cmd.toJsonValue();
		ClientCommandTouchback restored = new ClientCommandTouchback().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 8, restored.getEntropy());
		assertEquals(new FieldCoordinate(5, 6), restored.getBallCoordinate());
	}

	@Test
	public void roundTripWithNoCoordinate() {
		ClientCommandTouchback cmd = new ClientCommandTouchback();
		JsonObject json = cmd.toJsonValue();
		ClientCommandTouchback restored = new ClientCommandTouchback().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getBallCoordinate());
	}
}
