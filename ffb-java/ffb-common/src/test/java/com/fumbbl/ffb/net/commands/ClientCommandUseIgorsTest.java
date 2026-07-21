package com.fumbbl.ffb.net.commands;

import com.fumbbl.ffb.bb2020.InjuryDescription;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_use_igors.rs tests.
 * The Rust port stores injury descriptions as raw JSON strings; Java stores typed
 * {@link InjuryDescription} objects. Assertions are adapted to the Java API.
 */
public class ClientCommandUseIgorsTest {

	private static InjuryDescription injury(String playerId) {
		return new InjuryDescription(playerId, null, null, new ArrayList<>());
	}

	@Test
	public void defaultEmpty() {
		assertTrue(new ClientCommandUseIgors().getInjuryDescriptions().isEmpty());
	}

	@Test
	public void getterReturnsSlice() {
		ClientCommandUseIgors cmd = new ClientCommandUseIgors(Collections.singletonList(injury("p1")));
		assertEquals(1, cmd.getInjuryDescriptions().size());
	}

	@Test
	public void getIdIsClientUseIgors() {
		assertEquals(NetCommandId.CLIENT_USE_IGORS, new ClientCommandUseIgors().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndInjuryDescriptions() {
		ClientCommandUseIgors cmd = new ClientCommandUseIgors(Collections.singletonList(injury("p1")));
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientUseIgors", json.get("netCommandId").asString());
		assertEquals("p1", json.get("injuryDescriptions").asArray().get(0).asObject().get("playerId").asString());
	}

	@Test
	public void roundTripWithDescriptionsAndEntropy() {
		ClientCommandUseIgors cmd = new ClientCommandUseIgors(Collections.singletonList(injury("p1")));
		cmd.setEntropy((byte) 7);
		JsonObject json = cmd.toJsonValue();
		ClientCommandUseIgors restored = new ClientCommandUseIgors()
			.initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 7, restored.getEntropy());
		List<InjuryDescription> descriptions = restored.getInjuryDescriptions();
		assertEquals(1, descriptions.size());
		assertEquals("p1", descriptions.get(0).getPlayerId());
	}

	@Test
	public void roundTripWithNoDescriptions() {
		ClientCommandUseIgors cmd = new ClientCommandUseIgors();
		JsonObject json = cmd.toJsonValue();
		ClientCommandUseIgors restored = new ClientCommandUseIgors()
			.initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.getInjuryDescriptions().isEmpty());
	}
}
