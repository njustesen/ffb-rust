package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.FactoryType.Factory;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.SeriousInjury;
import com.fumbbl.ffb.factory.SeriousInjuryFactory;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_apothecary_choice.rs tests.
 *
 * The Rust SeriousInjuryKind::Dead maps to the game's real SeriousInjury "Dead (RIP)" obtained
 * from the SeriousInjuryFactory so that the value round-trips through the same factory on the wire.
 */
public class ClientCommandApothecaryChoiceTest {

	private SeriousInjury dead() {
		SeriousInjuryFactory factory = NetCommandTestUtil.gameSource().getFactory(Factory.SERIOUS_INJURY);
		return factory.dead();
	}

	@Test
	public void defaultAllNone() {
		ClientCommandApothecaryChoice cmd = new ClientCommandApothecaryChoice();
		assertNull(cmd.getPlayerId());
		assertNull(cmd.getPlayerState());
		assertNull(cmd.getOldPlayerState());
		assertNull(cmd.getSeriousInjury());
	}

	@Test
	public void fieldsAccessible() {
		SeriousInjury dead = dead();
		ClientCommandApothecaryChoice cmd = new ClientCommandApothecaryChoice("p1", null, dead, null);
		assertEquals("p1", cmd.getPlayerId());
		assertEquals(dead, cmd.getSeriousInjury());
	}

	@Test
	public void getIdIsClientApothecaryChoice() {
		assertEquals(NetCommandId.CLIENT_APOTHECARY_CHOICE, new ClientCommandApothecaryChoice().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndPlayerState() {
		ClientCommandApothecaryChoice cmd = new ClientCommandApothecaryChoice(null, new PlayerState(3), null, null);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientApothecaryChoice", json.get("netCommandId").asString());
		assertEquals(3, json.get("playerState").asInt());
	}

	@Test
	public void roundTripWithPopulatedFields() {
		SeriousInjury dead = dead();
		ClientCommandApothecaryChoice cmd = new ClientCommandApothecaryChoice("p1", new PlayerState(7), dead, new PlayerState(2));
		cmd.setEntropy((byte) 6);
		JsonObject json = cmd.toJsonValue();
		ClientCommandApothecaryChoice restored = new ClientCommandApothecaryChoice().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 6, restored.getEntropy());
		assertEquals("p1", restored.getPlayerId());
		assertNotNull(restored.getPlayerState());
		assertEquals(7, restored.getPlayerState().getId());
		assertNotNull(restored.getOldPlayerState());
		assertEquals(2, restored.getOldPlayerState().getId());
		assertEquals(dead, restored.getSeriousInjury());
	}

	@Test
	public void roundTripWithDefaultData() {
		ClientCommandApothecaryChoice cmd = new ClientCommandApothecaryChoice();
		JsonObject json = cmd.toJsonValue();
		ClientCommandApothecaryChoice restored = new ClientCommandApothecaryChoice().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getPlayerId());
		assertNull(restored.getPlayerState());
		assertNull(restored.getOldPlayerState());
		assertNull(restored.getSeriousInjury());
	}
}
