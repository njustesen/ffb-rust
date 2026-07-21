package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_kickoff.rs tests.
 */
public class ClientCommandKickoffTest {

	@Test
	public void coordinateStored() {
		FieldCoordinate coord = new FieldCoordinate(7, 3);
		ClientCommandKickoff cmd = new ClientCommandKickoff(coord);
		assertEquals(coord, cmd.getBallCoordinate());
	}

	@Test
	public void defaultHasNoCoordinate() {
		ClientCommandKickoff cmd = new ClientCommandKickoff();
		assertNull(cmd.getBallCoordinate());
	}

	@Test
	public void getIdIsClientKickoff() {
		assertEquals(NetCommandId.CLIENT_KICKOFF, new ClientCommandKickoff().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndBallCoordinate() {
		ClientCommandKickoff cmd = new ClientCommandKickoff(new FieldCoordinate(2, 9));
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientKickoff", json.get("netCommandId").asString());
		assertEquals(2, json.get("ballCoordinate").asArray().get(0).asInt());
		assertEquals(9, json.get("ballCoordinate").asArray().get(1).asInt());
	}

	@Test
	public void roundTripWithCoordinateAndEntropy() {
		ClientCommandKickoff cmd = new ClientCommandKickoff(new FieldCoordinate(5, 6));
		cmd.setEntropy((byte) 2);
		JsonObject json = cmd.toJsonValue();
		ClientCommandKickoff restored = new ClientCommandKickoff().initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 2, restored.getEntropy());
		assertEquals(new FieldCoordinate(5, 6), restored.getBallCoordinate());
	}

	@Test
	public void roundTripWithNoCoordinate() {
		ClientCommandKickoff cmd = new ClientCommandKickoff();
		JsonObject json = cmd.toJsonValue();
		ClientCommandKickoff restored = new ClientCommandKickoff().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getBallCoordinate());
	}
}
