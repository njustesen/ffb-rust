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
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_throw_team_mate.rs tests.
 *
 * NOTE: The Rust struct's {@code new(...)} constructor sets targetCoordinate, thrownPlayerId AND
 * actingPlayerId simultaneously. Java offers only mutually-exclusive constructors: either
 * (actingPlayerId, thrownPlayerId, kicked) OR (actingPlayerId, targetCoordinate, kicked) — never
 * both a coordinate and a thrownPlayerId at once. The tests below use the coordinate-based
 * constructor and adapt assertions accordingly (thrownPlayerId stays null on construction).
 */
public class ClientCommandThrowTeamMateTest {

	@Test
	public void allFieldsStored() {
		FieldCoordinate coord = new FieldCoordinate(5, 8);
		ClientCommandThrowTeamMate cmd = new ClientCommandThrowTeamMate("thrower1", coord, true);
		assertEquals(coord, cmd.getTargetCoordinate());
		assertEquals("thrower1", cmd.getActingPlayerId());
		assertTrue(cmd.isKicked());
		assertNull(cmd.getThrownPlayerId());
	}

	@Test
	public void defaultIsEmpty() {
		ClientCommandThrowTeamMate cmd = new ClientCommandThrowTeamMate();
		assertNull(cmd.getTargetCoordinate());
		assertNull(cmd.getThrownPlayerId());
		assertFalse(cmd.isKicked());
	}

	@Test
	public void getIdIsClientThrowTeamMate() {
		assertEquals(NetCommandId.CLIENT_THROW_TEAM_MATE, new ClientCommandThrowTeamMate().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndKicked() {
		ClientCommandThrowTeamMate cmd = new ClientCommandThrowTeamMate("a", new FieldCoordinate(1, 1), true);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientThrowTeamMate", json.get("netCommandId").asString());
		assertTrue(json.get("kicked").asBoolean());
	}

	@Test
	public void roundTripWithAllFieldsAndEntropy() {
		FieldCoordinate coord = new FieldCoordinate(9, 10);
		ClientCommandThrowTeamMate cmd = new ClientCommandThrowTeamMate("thrower2", coord, false);
		cmd.setEntropy((byte) 21);
		JsonObject json = cmd.toJsonValue();
		ClientCommandThrowTeamMate restored = new ClientCommandThrowTeamMate().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 21, restored.getEntropy());
		assertEquals(coord, restored.getTargetCoordinate());
		assertEquals("thrower2", restored.getActingPlayerId());
		assertFalse(restored.isKicked());
	}

	@Test
	public void roundTripWithNoCoordinateOrIds() {
		ClientCommandThrowTeamMate cmd = new ClientCommandThrowTeamMate();
		JsonObject json = cmd.toJsonValue();
		ClientCommandThrowTeamMate restored = new ClientCommandThrowTeamMate().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getTargetCoordinate());
		assertNull(restored.getThrownPlayerId());
		assertNull(restored.getActingPlayerId());
		assertFalse(restored.isKicked());
	}
}
