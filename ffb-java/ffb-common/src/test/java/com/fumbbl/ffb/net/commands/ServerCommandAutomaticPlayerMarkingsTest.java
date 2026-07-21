package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import java.util.HashMap;
import java.util.Map;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/server_command_automatic_player_markings.rs tests.
 */
public class ServerCommandAutomaticPlayerMarkingsTest {

	@Test
	public void fieldsStored() {
		Map<String, String> markings = new HashMap<>();
		markings.put("p1", "red");
		ServerCommandAutomaticPlayerMarkings cmd = new ServerCommandAutomaticPlayerMarkings(2, markings);
		assertEquals(2, cmd.getIndex());
		assertEquals("red", cmd.getMarkings().get("p1"));
	}

	@Test
	public void defaultEmpty() {
		// The Rust default() yields an empty map; the Java no-arg constructor
		// leaves the map null.
		ServerCommandAutomaticPlayerMarkings cmd = new ServerCommandAutomaticPlayerMarkings();
		assertNull(cmd.getMarkings());
		assertEquals(0, cmd.getIndex());
	}

	@Test
	public void getIdIsServerAutomaticPlayerMarkings() {
		assertEquals(NetCommandId.SERVER_AUTOMATIC_PLAYER_MARKINGS, new ServerCommandAutomaticPlayerMarkings().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndSelectedIndex() {
		Map<String, String> markings = new HashMap<>();
		markings.put("p1", "red");
		ServerCommandAutomaticPlayerMarkings cmd = new ServerCommandAutomaticPlayerMarkings(3, markings);
		JsonObject json = cmd.toJsonValue().asObject();
		assertEquals("serverAutomaticPlayerMarkings", json.get("netCommandId").asString());
		assertEquals(3, json.get("selectedIndex").asInt());
		assertEquals("red", json.get("markings").asObject().get("p1").asString());
	}

	@Test
	public void roundTripWithMarkings() {
		Map<String, String> markings = new HashMap<>();
		markings.put("p1", "red");
		markings.put("p2", "blue");
		ServerCommandAutomaticPlayerMarkings cmd = new ServerCommandAutomaticPlayerMarkings(1, markings);
		cmd.setCommandNr(7);
		JsonObject json = cmd.toJsonValue().asObject();
		ServerCommandAutomaticPlayerMarkings restored =
			(ServerCommandAutomaticPlayerMarkings) new ServerCommandAutomaticPlayerMarkings()
				.initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(7, restored.getCommandNr());
		assertEquals(1, restored.getIndex());
		assertEquals("red", restored.getMarkings().get("p1"));
		assertEquals("blue", restored.getMarkings().get("p2"));
	}

	@Test
	public void roundTripWithNoMarkings() {
		ServerCommandAutomaticPlayerMarkings cmd = new ServerCommandAutomaticPlayerMarkings(0, new HashMap<>());
		JsonObject json = cmd.toJsonValue().asObject();
		ServerCommandAutomaticPlayerMarkings restored =
			(ServerCommandAutomaticPlayerMarkings) new ServerCommandAutomaticPlayerMarkings()
				.initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.getMarkings().isEmpty());
		assertEquals(0, restored.getIndex());
	}
}
