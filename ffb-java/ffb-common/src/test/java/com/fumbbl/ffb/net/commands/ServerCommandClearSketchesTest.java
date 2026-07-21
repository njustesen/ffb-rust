package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/server_command_clear_sketches.rs tests.
 */
public class ServerCommandClearSketchesTest {

	@Test
	public void canBeCreated() {
		assertNotNull(new ServerCommandClearSketches());
	}

	@Test
	public void getIdIsServerClearSketches() {
		assertEquals(NetCommandId.SERVER_CLEAR_SKETCHES, new ServerCommandClearSketches().getId());
	}

	@Test
	public void toJsonValueHasOnlyNetCommandId() {
		JsonObject json = new ServerCommandClearSketches().toJsonValue();
		assertEquals("serverClearSketches", json.get("netCommandId").asString());
		assertNull(json.get("commandNr"));
	}

	@Test
	public void roundTrip() {
		ServerCommandClearSketches cmd = new ServerCommandClearSketches();
		JsonObject json = cmd.toJsonValue();
		ServerCommandClearSketches restored = new ServerCommandClearSketches().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNotNull(restored);
	}
}
