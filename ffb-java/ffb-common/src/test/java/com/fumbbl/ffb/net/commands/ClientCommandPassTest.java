package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_pass.rs tests.
 */
public class ClientCommandPassTest {

	@Test
	public void fieldsStored() {
		FieldCoordinate coord = new FieldCoordinate(10, 5);
		ClientCommandPass cmd = new ClientCommandPass("thrower1", coord);
		assertEquals("thrower1", cmd.getActingPlayerId());
		assertEquals(coord, cmd.getTargetCoordinate());
	}

	@Test
	public void defaultIsEmpty() {
		ClientCommandPass cmd = new ClientCommandPass();
		assertNull(cmd.getActingPlayerId());
		assertNull(cmd.getTargetCoordinate());
	}

	@Test
	public void newWithCoord() {
		FieldCoordinate coord = new FieldCoordinate(0, 0);
		ClientCommandPass cmd = new ClientCommandPass("p1", coord);
		assertNotNull(cmd.getActingPlayerId());
	}

	@Test
	public void getIdIsClientPass() {
		assertEquals(NetCommandId.CLIENT_PASS, new ClientCommandPass().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndTargetCoordinate() {
		ClientCommandPass cmd = new ClientCommandPass("p1", new FieldCoordinate(9, 1));
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientPass", json.get("netCommandId").asString());
		assertEquals(9, json.get("targetCoordinate").asArray().get(0).asInt());
		assertEquals(1, json.get("targetCoordinate").asArray().get(1).asInt());
	}

	@Test
	public void roundTripWithData() {
		ClientCommandPass cmd = new ClientCommandPass("p2", new FieldCoordinate(4, 4));
		cmd.setEntropy((byte) 9);
		JsonObject json = cmd.toJsonValue();
		ClientCommandPass restored = new ClientCommandPass().initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 9, restored.getEntropy());
		assertEquals("p2", restored.getActingPlayerId());
		assertEquals(new FieldCoordinate(4, 4), restored.getTargetCoordinate());
	}

	@Test
	public void roundTripDefault() {
		ClientCommandPass cmd = new ClientCommandPass();
		JsonObject json = cmd.toJsonValue();
		ClientCommandPass restored = new ClientCommandPass().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getActingPlayerId());
		assertNull(restored.getTargetCoordinate());
		assertFalse(restored.hasEntropy());
	}
}
