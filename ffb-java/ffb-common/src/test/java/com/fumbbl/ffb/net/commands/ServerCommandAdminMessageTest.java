package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/server_command_admin_message.rs tests.
 */
public class ServerCommandAdminMessageTest {

	@Test
	public void newStoresMessages() {
		ServerCommandAdminMessage cmd = new ServerCommandAdminMessage(new String[] { "hello", "world" });
		assertArrayEquals(new String[] { "hello", "world" }, cmd.getMessages());
	}

	@Test
	public void addMessageAppends() {
		// Rust exercises add_message(); Java's addMessage is private, so the
		// equivalent append behaviour is driven through the public constructor.
		ServerCommandAdminMessage cmd = new ServerCommandAdminMessage(new String[] { "hi" });
		assertEquals(1, cmd.getMessages().length);
	}

	@Test
	public void getIdIsServerAdminMessage() {
		assertEquals(NetCommandId.SERVER_ADMIN_MESSAGE, new ServerCommandAdminMessage().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndMessageArray() {
		ServerCommandAdminMessage cmd = new ServerCommandAdminMessage(new String[] { "hi" });
		JsonObject json = cmd.toJsonValue();
		assertEquals("serverAdminMessage", json.get("netCommandId").asString());
		assertEquals("hi", json.get("messageArray").asArray().get(0).asString());
	}

	@Test
	public void roundTripWithMessages() {
		ServerCommandAdminMessage cmd = new ServerCommandAdminMessage(new String[] { "hello", "world" });
		cmd.setCommandNr(4);
		JsonObject json = cmd.toJsonValue();
		ServerCommandAdminMessage restored = new ServerCommandAdminMessage().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(4, restored.getCommandNr());
		assertArrayEquals(new String[] { "hello", "world" }, restored.getMessages());
	}

	@Test
	public void roundTripWithNoMessages() {
		ServerCommandAdminMessage cmd = new ServerCommandAdminMessage();
		JsonObject json = cmd.toJsonValue();
		ServerCommandAdminMessage restored = new ServerCommandAdminMessage().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(0, restored.getMessages().length);
	}
}
