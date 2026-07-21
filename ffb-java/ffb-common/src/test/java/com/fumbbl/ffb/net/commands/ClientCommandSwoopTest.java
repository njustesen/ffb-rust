package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_swoop.rs tests.
 */
public class ClientCommandSwoopTest {

	@Test
	public void fieldsStored() {
		FieldCoordinate coord = new FieldCoordinate(7, 3);
		ClientCommandSwoop cmd = new ClientCommandSwoop("attacker1", coord);
		assertEquals("attacker1", cmd.getActingPlayerId());
		assertEquals(coord, cmd.getTargetCoordinate());
	}

	@Test
	public void defaultIsEmpty() {
		ClientCommandSwoop cmd = new ClientCommandSwoop();
		assertNull(cmd.getActingPlayerId());
		assertNull(cmd.getTargetCoordinate());
	}

	@Test
	public void getIdIsClientSwoop() {
		assertEquals(NetCommandId.CLIENT_SWOOP, new ClientCommandSwoop().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndActingPlayerId() {
		ClientCommandSwoop cmd = new ClientCommandSwoop("p1", new FieldCoordinate(2, 5));
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientSwoop", json.get("netCommandId").asString());
		assertEquals("p1", json.get("actingPlayerId").asString());
	}

	@Test
	public void roundTripWithAllFields() {
		ClientCommandSwoop cmd = new ClientCommandSwoop("p1", new FieldCoordinate(2, 5));
		cmd.setEntropy((byte) 11);
		JsonObject json = cmd.toJsonValue();
		ClientCommandSwoop restored = new ClientCommandSwoop().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 11, restored.getEntropy());
		assertEquals("p1", restored.getActingPlayerId());
		assertEquals(new FieldCoordinate(2, 5), restored.getTargetCoordinate());
	}

	@Test
	public void roundTripWithNoFields() {
		ClientCommandSwoop cmd = new ClientCommandSwoop();
		JsonObject json = cmd.toJsonValue();
		ClientCommandSwoop restored = new ClientCommandSwoop().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getActingPlayerId());
		assertNull(restored.getTargetCoordinate());
	}
}
