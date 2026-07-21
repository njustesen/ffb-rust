package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.bb2020.InjuryDescription;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_use_apothecaries.rs tests.
 *
 * The Rust struct defers InjuryDescription to raw JSON strings; Java stores real
 * {@link InjuryDescription} objects, so these tests construct InjuryDescription instances directly.
 */
public class ClientCommandUseApothecariesTest {

	private static InjuryDescription injury(String playerId) {
		return new InjuryDescription(playerId, null, null, new ArrayList<>());
	}

	@Test
	public void defaultHasEmptyDescriptions() {
		ClientCommandUseApothecaries cmd = new ClientCommandUseApothecaries();
		assertTrue(cmd.getInjuryDescriptions().isEmpty());
	}

	@Test
	public void storesInjuryDescriptions() {
		List<InjuryDescription> list = Collections.singletonList(injury("p1"));
		ClientCommandUseApothecaries cmd = new ClientCommandUseApothecaries(list);
		assertEquals(1, cmd.getInjuryDescriptions().size());
	}

	@Test
	public void getIdIsClientUseApothecaries() {
		assertEquals(NetCommandId.CLIENT_USE_APOTHECARIES, new ClientCommandUseApothecaries().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndInjuryDescriptions() {
		List<InjuryDescription> list = Collections.singletonList(injury("p1"));
		ClientCommandUseApothecaries cmd = new ClientCommandUseApothecaries(list);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientUseApothecaries", json.get("netCommandId").asString());
		assertEquals("p1", json.get("injuryDescriptions").asArray().get(0).asObject().get("playerId").asString());
	}

	@Test
	public void roundTripWithDescriptionsAndEntropy() {
		List<InjuryDescription> list = Collections.singletonList(injury("p2"));
		ClientCommandUseApothecaries cmd = new ClientCommandUseApothecaries(list);
		cmd.setEntropy((byte) 9);
		JsonObject json = cmd.toJsonValue();
		ClientCommandUseApothecaries restored = new ClientCommandUseApothecaries().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 9, restored.getEntropy());
		assertEquals(1, restored.getInjuryDescriptions().size());
		assertEquals("p2", restored.getInjuryDescriptions().get(0).getPlayerId());
	}

	@Test
	public void roundTripWithEmptyDescriptions() {
		ClientCommandUseApothecaries cmd = new ClientCommandUseApothecaries();
		JsonObject json = cmd.toJsonValue();
		ClientCommandUseApothecaries restored = new ClientCommandUseApothecaries().initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.getInjuryDescriptions().isEmpty());
	}
}
