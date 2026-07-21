package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_position_selection.rs tests.
 * The Java constructor takes (String[] position, String teamId).
 */
public class ClientCommandPositionSelectionTest {

	@Test
	public void fieldsStored() {
		ClientCommandPositionSelection cmd =
			new ClientCommandPositionSelection(new String[] { "Lineman", "Blitzer" }, "team1");
		assertEquals("team1", cmd.getTeamId());
		assertEquals(2, cmd.getPosition().length);
	}

	@Test
	public void defaultIsEmpty() {
		ClientCommandPositionSelection cmd = new ClientCommandPositionSelection();
		assertNull(cmd.getTeamId());
		assertNull(cmd.getPosition());
	}

	@Test
	public void positionSliceMatchesInput() {
		String[] positions = { "Lineman" };
		ClientCommandPositionSelection cmd = new ClientCommandPositionSelection(positions, "t1");
		assertEquals(1, cmd.getPosition().length);
		assertEquals("Lineman", cmd.getPosition()[0]);
	}

	@Test
	public void getIdIsClientPositionSelection() {
		assertEquals(NetCommandId.CLIENT_POSITION_SELECTION, new ClientCommandPositionSelection().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndPositionIds() {
		ClientCommandPositionSelection cmd =
			new ClientCommandPositionSelection(new String[] { "Blitzer" }, "t1");
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientPositionSelection", json.get("netCommandId").asString());
		assertEquals(1, json.get("positionIds").asArray().size());
		assertEquals("Blitzer", json.get("positionIds").asArray().get(0).asString());
		assertEquals("t1", json.get("teamId").asString());
	}

	@Test
	public void roundTripWithData() {
		ClientCommandPositionSelection cmd =
			new ClientCommandPositionSelection(new String[] { "Lineman", "Ogre" }, "t2");
		cmd.setEntropy((byte) 1);
		JsonObject json = cmd.toJsonValue();
		ClientCommandPositionSelection restored =
			(ClientCommandPositionSelection) new ClientCommandPositionSelection().initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 1, restored.getEntropy());
		assertEquals("t2", restored.getTeamId());
		assertEquals(2, restored.getPosition().length);
		assertEquals("Lineman", restored.getPosition()[0]);
		assertEquals("Ogre", restored.getPosition()[1]);
	}

	@Test
	public void roundTripDefault() {
		ClientCommandPositionSelection cmd = new ClientCommandPositionSelection();
		JsonObject json = cmd.toJsonValue();
		ClientCommandPositionSelection restored =
			(ClientCommandPositionSelection) new ClientCommandPositionSelection().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getPosition());
		assertNull(restored.getTeamId());
		assertFalse(restored.hasEntropy());
	}
}
