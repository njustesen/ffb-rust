package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.FactoryType.FactoryContext;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/server_command_version.rs tests.
 */
public class ServerCommandVersionTest {

	@Test
	public void fieldsStored() {
		ServerCommandVersion cmd = new ServerCommandVersion("1.2.3", "1.0.0", new String[0], new String[0], false);
		assertEquals("1.2.3", cmd.getServerVersion());
		assertEquals("1.0.0", cmd.getClientVersion());
		assertFalse(cmd.isTestServer());
	}

	@Test
	public void defaultEmpty() {
		ServerCommandVersion cmd = new ServerCommandVersion();
		assertNull(cmd.getServerVersion());
		assertEquals(0, cmd.getClientProperties().length);
	}

	@Test
	public void getIdIsServerVersion() {
		assertEquals(NetCommandId.SERVER_VERSION, new ServerCommandVersion().getId());
	}

	@Test
	public void getContextIsApplication() {
		assertEquals(FactoryContext.APPLICATION, new ServerCommandVersion().getContext());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndVersions() {
		ServerCommandVersion cmd = new ServerCommandVersion("1.2.3", "1.0.0", new String[0], new String[0], true);
		JsonObject json = cmd.toJsonValue();
		assertEquals("serverVersion", json.get("netCommandId").asString());
		assertEquals("1.2.3", json.get("serverVersion").asString());
		assertEquals("1.0.0", json.get("clientVersion").asString());
		assertTrue(json.get("testing").asBoolean());
	}

	@Test
	public void roundTripWithProperties() {
		ServerCommandVersion cmd = new ServerCommandVersion("1.2.3", "1.0.0", new String[] { "os" }, new String[] { "windows" }, true);
		cmd.setCommandNr(7);
		JsonObject json = cmd.toJsonValue();
		ServerCommandVersion restored = new ServerCommandVersion().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(7, restored.getCommandNr());
		assertEquals("1.2.3", restored.getServerVersion());
		assertEquals("1.0.0", restored.getClientVersion());
		assertTrue(restored.isTestServer());
		assertEquals("windows", restored.getClientPropertyValue("os"));
	}

	@Test
	public void roundTripWithDefault() {
		ServerCommandVersion cmd = new ServerCommandVersion();
		JsonObject json = cmd.toJsonValue();
		ServerCommandVersion restored = new ServerCommandVersion().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getServerVersion());
		assertEquals(0, restored.getClientProperties().length);
		assertFalse(restored.isTestServer());
	}
}
