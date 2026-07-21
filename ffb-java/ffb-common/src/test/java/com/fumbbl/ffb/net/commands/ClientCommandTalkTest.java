package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_talk.rs tests.
 */
public class ClientCommandTalkTest {

	@Test
	public void defaultHasNoTalk() {
		ClientCommandTalk cmd = new ClientCommandTalk();
		assertNull(cmd.getTalk());
	}

	@Test
	public void withTalkStoresValue() {
		ClientCommandTalk cmd = new ClientCommandTalk("hello");
		assertEquals("hello", cmd.getTalk());
	}

	@Test
	public void getIdIsClientTalk() {
		assertEquals(NetCommandId.CLIENT_TALK, new ClientCommandTalk().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndTalk() {
		ClientCommandTalk cmd = new ClientCommandTalk("hi there");
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientTalk", json.get("netCommandId").asString());
		assertEquals("hi there", json.get("talk").asString());
	}

	@Test
	public void roundTripWithTalkAndEntropy() {
		ClientCommandTalk cmd = new ClientCommandTalk("gg");
		cmd.setEntropy((byte) 1);
		JsonObject json = cmd.toJsonValue();
		ClientCommandTalk restored = new ClientCommandTalk().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 1, restored.getEntropy());
		assertEquals("gg", restored.getTalk());
	}

	@Test
	public void roundTripWithNoTalk() {
		ClientCommandTalk cmd = new ClientCommandTalk();
		JsonObject json = cmd.toJsonValue();
		ClientCommandTalk restored = new ClientCommandTalk().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getTalk());
	}
}
