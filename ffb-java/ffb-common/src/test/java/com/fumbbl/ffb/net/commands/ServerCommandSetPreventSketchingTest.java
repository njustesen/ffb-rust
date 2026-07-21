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
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/server_command_set_prevent_sketching.rs tests.
 */
public class ServerCommandSetPreventSketchingTest {

	@Test
	public void flagStored() {
		ServerCommandSetPreventSketching cmd = new ServerCommandSetPreventSketching("Alice", true);
		assertTrue(cmd.isPreventSketching());
	}

	@Test
	public void defaultAllow() {
		ServerCommandSetPreventSketching cmd = new ServerCommandSetPreventSketching();
		assertFalse(cmd.isPreventSketching());
	}

	@Test
	public void getIdIsServerSetPreventSketching() {
		assertEquals(NetCommandId.SERVER_SET_PREVENT_SKETCHING, new ServerCommandSetPreventSketching().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndFields() {
		ServerCommandSetPreventSketching cmd = new ServerCommandSetPreventSketching("Alice", true);
		JsonObject json = cmd.toJsonValue();
		assertEquals("serverSetPreventSketching", json.get("netCommandId").asString());
		assertTrue(json.get("prevent").asBoolean());
		assertEquals("Alice", json.get("coach").asString());
	}

	@Test
	public void roundTripWithData() {
		ServerCommandSetPreventSketching cmd = new ServerCommandSetPreventSketching("Bob", true);
		JsonObject json = cmd.toJsonValue();
		ServerCommandSetPreventSketching restored = new ServerCommandSetPreventSketching().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals("Bob", restored.getCoach());
		assertTrue(restored.isPreventSketching());
	}

	@Test
	public void roundTripWithDefault() {
		ServerCommandSetPreventSketching cmd = new ServerCommandSetPreventSketching();
		JsonObject json = cmd.toJsonValue();
		ServerCommandSetPreventSketching restored = new ServerCommandSetPreventSketching().initFrom(NetCommandTestUtil.gameSource(), json);
		// Java restores null coach (Rust default() is empty string).
		assertNull(restored.getCoach());
		assertFalse(restored.isPreventSketching());
	}
}
