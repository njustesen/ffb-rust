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
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_set_prevent_sketching.rs tests.
 */
public class ClientCommandSetPreventSketchingTest {

	@Test
	public void defaultHasNoCoachAndFalseFlag() {
		ClientCommandSetPreventSketching cmd = new ClientCommandSetPreventSketching();
		assertNull(cmd.getCoach());
		assertFalse(cmd.isPreventSketching());
	}

	@Test
	public void withFieldsStoresValues() {
		ClientCommandSetPreventSketching cmd = new ClientCommandSetPreventSketching("coach-1", true);
		assertEquals("coach-1", cmd.getCoach());
		assertTrue(cmd.isPreventSketching());
	}

	@Test
	public void falsePreventStored() {
		ClientCommandSetPreventSketching cmd = new ClientCommandSetPreventSketching("c", false);
		assertFalse(cmd.isPreventSketching());
	}

	@Test
	public void getIdIsClientSetPreventSketching() {
		assertEquals(NetCommandId.CLIENT_SET_PREVENT_SKETCHING, new ClientCommandSetPreventSketching().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndPrevent() {
		ClientCommandSetPreventSketching cmd = new ClientCommandSetPreventSketching("coach-1", true);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientSetPreventSketching", json.get("netCommandId").asString());
		assertTrue(json.get("prevent").asBoolean());
		assertEquals("coach-1", json.get("coach").asString());
	}

	@Test
	public void roundTripPopulated() {
		ClientCommandSetPreventSketching cmd = new ClientCommandSetPreventSketching("coach-2", true);
		cmd.setEntropy((byte) 1);
		JsonObject json = cmd.toJsonValue();
		ClientCommandSetPreventSketching restored = new ClientCommandSetPreventSketching().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals("coach-2", restored.getCoach());
		assertTrue(restored.isPreventSketching());
		assertEquals((byte) 1, restored.getEntropy());
	}

	@Test
	public void roundTripDefault() {
		ClientCommandSetPreventSketching cmd = new ClientCommandSetPreventSketching();
		JsonObject json = cmd.toJsonValue();
		ClientCommandSetPreventSketching restored = new ClientCommandSetPreventSketching().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getCoach());
		assertFalse(restored.isPreventSketching());
		assertFalse(restored.hasEntropy());
	}
}
