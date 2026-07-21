package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.ApothecaryType;
import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.SeriousInjury;
import com.fumbbl.ffb.factory.SeriousInjuryFactory;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_use_apothecary.rs tests.
 *
 * The Rust struct simplifies seriousInjury to an enum kind; Java stores a factory-backed
 * SeriousInjury object. Tests obtain a real SeriousInjury via the game's SeriousInjuryFactory.
 */
public class ClientCommandUseApothecaryTest {

	private static SeriousInjury deadInjury() {
		SeriousInjuryFactory factory = NetCommandTestUtil.gameSource().getFactory(FactoryType.Factory.SERIOUS_INJURY);
		return factory.dead();
	}

	@Test
	public void defaultApothecaryNotUsed() {
		ClientCommandUseApothecary cmd = new ClientCommandUseApothecary();
		assertFalse(cmd.isApothecaryUsed());
		assertNull(cmd.getPlayerId());
	}

	@Test
	public void storesApothecaryFields() {
		SeriousInjury injury = deadInjury();
		ClientCommandUseApothecary cmd = new ClientCommandUseApothecary("player_3", true, ApothecaryType.TEAM, injury,
			new PlayerState(1));
		assertEquals("player_3", cmd.getPlayerId());
		assertTrue(cmd.isApothecaryUsed());
		assertEquals(ApothecaryType.TEAM, cmd.getApothecaryType());
		assertEquals(injury, cmd.getSeriousInjury());
	}

	@Test
	public void getIdIsClientUseApothecary() {
		assertEquals(NetCommandId.CLIENT_USE_APOTHECARY, new ClientCommandUseApothecary().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndApothecaryUsed() {
		ClientCommandUseApothecary cmd = new ClientCommandUseApothecary("p1", true, null, null);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientUseApothecary", json.get("netCommandId").asString());
		assertTrue(json.get("apothecaryUsed").asBoolean());
		assertEquals("p1", json.get("playerId").asString());
	}

	@Test
	public void roundTripWithAllFieldsAndEntropy() {
		SeriousInjury injury = deadInjury();
		ClientCommandUseApothecary cmd = new ClientCommandUseApothecary("player_4", true, ApothecaryType.WANDERING,
			injury, new PlayerState(7));
		cmd.setEntropy((byte) 17);
		JsonObject json = cmd.toJsonValue();
		ClientCommandUseApothecary restored = new ClientCommandUseApothecary().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 17, restored.getEntropy());
		assertEquals("player_4", restored.getPlayerId());
		assertTrue(restored.isApothecaryUsed());
		assertEquals(ApothecaryType.WANDERING, restored.getApothecaryType());
		assertEquals(injury, restored.getSeriousInjury());
		assertEquals(new PlayerState(7), restored.getPlayerState());
	}

	@Test
	public void roundTripWithNoOptionalFields() {
		ClientCommandUseApothecary cmd = new ClientCommandUseApothecary();
		JsonObject json = cmd.toJsonValue();
		assertNull(json.get("apothecaryType"));
		assertNull(json.get("seriousInjury"));
		assertNull(json.get("playerState"));
		ClientCommandUseApothecary restored = new ClientCommandUseApothecary().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getApothecaryType());
		assertNull(restored.getSeriousInjury());
		assertNull(restored.getPlayerState());
	}
}
