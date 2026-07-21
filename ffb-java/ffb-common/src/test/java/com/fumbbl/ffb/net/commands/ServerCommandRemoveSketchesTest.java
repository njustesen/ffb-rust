package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import java.util.ArrayList;
import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/server_command_remove_sketches.rs tests.
 */
public class ServerCommandRemoveSketchesTest {

	@Test
	public void fieldsStored() {
		ServerCommandRemoveSketches cmd = new ServerCommandRemoveSketches("Bob", Arrays.asList("id1"));
		assertEquals("Bob", cmd.getCoach());
		assertEquals(Arrays.asList("id1"), cmd.getIds());
	}

	@Test
	public void defaultEmpty() {
		ServerCommandRemoveSketches cmd = new ServerCommandRemoveSketches();
		assertNull(cmd.getIds());
	}

	@Test
	public void getIdIsServerRemoveSketches() {
		assertEquals(NetCommandId.SERVER_REMOVE_SKETCHES, new ServerCommandRemoveSketches().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndIds() {
		ServerCommandRemoveSketches cmd = new ServerCommandRemoveSketches("Bob", Arrays.asList("id1"));
		JsonObject json = cmd.toJsonValue();
		assertEquals("serverRemoveSketches", json.get("netCommandId").asString());
		assertEquals("Bob", json.get("coach").asString());
		assertEquals("id1", json.get("ids").asArray().get(0).asString());
	}

	@Test
	public void toJsonValueOmitsIdsWhenEmpty() {
		ServerCommandRemoveSketches cmd = new ServerCommandRemoveSketches("Bob", new ArrayList<>());
		JsonObject json = cmd.toJsonValue();
		assertNull(json.get("ids"));
	}

	@Test
	public void roundTripWithData() {
		ServerCommandRemoveSketches cmd = new ServerCommandRemoveSketches("Carol", Arrays.asList("a", "b"));
		JsonObject json = cmd.toJsonValue();
		ServerCommandRemoveSketches restored = new ServerCommandRemoveSketches().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals("Carol", restored.getCoach());
		assertEquals(Arrays.asList("a", "b"), restored.getIds());
	}

	@Test
	public void roundTripWithDefault() {
		ServerCommandRemoveSketches cmd = new ServerCommandRemoveSketches();
		JsonObject json = cmd.toJsonValue();
		ServerCommandRemoveSketches restored = new ServerCommandRemoveSketches().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getCoach());
		assertNull(restored.getIds());
	}
}
