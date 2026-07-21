package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.eclipsesource.json.JsonValue;
import com.fumbbl.ffb.factory.IFactorySource;
import com.fumbbl.ffb.json.IJsonOption;
import com.fumbbl.ffb.json.UtilJson;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/server_command.rs tests.
 * ServerCommand is abstract in Java and (unlike ClientCommand) has no shared
 * toJsonValue/initFrom; every concrete subclass writes netCommandId + commandNr
 * by hand. The private subclass here reproduces exactly that standard pattern so
 * the base-class behavior (commandNr, isReplayable) can be exercised 1:1 with
 * the Rust base helpers.
 */
public class ServerCommandTest {

	private static class TestServerCommand extends ServerCommand {
		public NetCommandId getId() {
			return NetCommandId.SERVER_GAME_TIME;
		}

		public JsonObject toJsonValue() {
			JsonObject jsonObject = new JsonObject();
			IJsonOption.NET_COMMAND_ID.addTo(jsonObject, getId());
			IJsonOption.COMMAND_NR.addTo(jsonObject, getCommandNr());
			return jsonObject;
		}

		public TestServerCommand initFrom(IFactorySource source, JsonValue jsonValue) {
			JsonObject jsonObject = UtilJson.toJsonObject(jsonValue);
			setCommandNr(IJsonOption.COMMAND_NR.getFrom(source, jsonObject));
			return this;
		}
	}

	@Test
	public void defaultCommandNrIsZero() {
		assertEquals(0, new TestServerCommand().getCommandNr());
	}

	@Test
	public void withCommandNrSetsField() {
		TestServerCommand cmd = new TestServerCommand();
		cmd.setCommandNr(7);
		assertEquals(7, cmd.getCommandNr());
	}

	@Test
	public void isReplayableDefaultTrue() {
		assertTrue(new TestServerCommand().isReplayable());
	}

	@Test
	public void serdeRoundTrip() {
		TestServerCommand cmd = new TestServerCommand();
		cmd.setCommandNr(42);
		JsonObject json = cmd.toJsonValue();
		TestServerCommand back = new TestServerCommand().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(42, back.getCommandNr());
	}

	@Test
	public void baseJsonFieldsIncludesNetCommandIdAndCommandNr() {
		TestServerCommand cmd = new TestServerCommand();
		cmd.setCommandNr(5);
		JsonObject json = cmd.toJsonValue();
		assertEquals("serverGameTime", json.get("netCommandId").asString());
		assertEquals(5, json.get("commandNr").asInt());
	}

	@Test
	public void baseFromJsonRoundTrip() {
		TestServerCommand cmd = new TestServerCommand();
		cmd.setCommandNr(11);
		JsonObject json = cmd.toJsonValue();
		TestServerCommand restored = new TestServerCommand().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(11, restored.getCommandNr());
	}
}
