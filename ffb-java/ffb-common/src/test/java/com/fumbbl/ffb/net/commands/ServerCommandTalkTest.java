package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/server_command_talk.rs tests.
 */
public class ServerCommandTalkTest {

	@Test
	public void fieldsStored() {
		ServerCommandTalk cmd = new ServerCommandTalk("Alice", "hi", ServerCommandTalk.Mode.REGULAR);
		assertEquals("Alice", cmd.getCoach());
		assertArrayEquals(new String[] { "hi" }, cmd.getTalks());
		assertEquals(ServerCommandTalk.Mode.REGULAR, cmd.getMode());
	}

	@Test
	public void defaultEmpty() {
		ServerCommandTalk cmd = new ServerCommandTalk();
		assertNull(cmd.getCoach());
		assertEquals(0, cmd.getTalks().length);
	}

	@Test
	public void getIdIsServerTalk() {
		assertEquals(NetCommandId.SERVER_TALK, new ServerCommandTalk().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndCoach() {
		ServerCommandTalk cmd = new ServerCommandTalk("Alice", "hi", ServerCommandTalk.Mode.REGULAR);
		JsonObject json = cmd.toJsonValue();
		assertEquals("serverTalk", json.get("netCommandId").asString());
		assertEquals("Alice", json.get("coach").asString());
		assertEquals("REGULAR", json.get("talkMode").asString());
		assertNull(json.get("commandNr"));
	}

	@Test
	public void roundTripWithTalks() {
		ServerCommandTalk cmd = new ServerCommandTalk("Bob", "hi", ServerCommandTalk.Mode.STAFF);
		cmd.addTalks(new String[] { "there" });
		JsonObject json = cmd.toJsonValue();
		ServerCommandTalk restored = new ServerCommandTalk().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals("Bob", restored.getCoach());
		assertArrayEquals(new String[] { "hi", "there" }, restored.getTalks());
		assertEquals(ServerCommandTalk.Mode.STAFF, restored.getMode());
	}

	@Test
	public void roundTripWithDefault() {
		ServerCommandTalk cmd = new ServerCommandTalk();
		JsonObject json = cmd.toJsonValue();
		ServerCommandTalk restored = new ServerCommandTalk().initFrom(NetCommandTestUtil.gameSource(), json);
		// Java restores null coach (Rust default() is empty string).
		assertNull(restored.getCoach());
		assertEquals(0, restored.getTalks().length);
		assertEquals(ServerCommandTalk.Mode.REGULAR, restored.getMode());
	}
}
