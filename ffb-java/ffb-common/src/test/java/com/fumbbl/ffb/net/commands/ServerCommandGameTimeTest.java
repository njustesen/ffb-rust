package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.FactoryType.FactoryContext;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/server_command_game_time.rs tests.
 */
public class ServerCommandGameTimeTest {

	@Test
	public void fieldsStored() {
		ServerCommandGameTime cmd = new ServerCommandGameTime(60000, 30000);
		assertEquals(60000L, cmd.getGameTime());
		assertEquals(30000L, cmd.getTurnTime());
	}

	@Test
	public void defaultZeros() {
		ServerCommandGameTime cmd = new ServerCommandGameTime();
		assertEquals(0L, cmd.getGameTime());
		assertEquals(0L, cmd.getTurnTime());
	}

	@Test
	public void getIdIsServerGameTime() {
		assertEquals(NetCommandId.SERVER_GAME_TIME, new ServerCommandGameTime().getId());
	}

	@Test
	public void getContextIsApplication() {
		assertEquals(FactoryContext.APPLICATION, new ServerCommandGameTime().getContext());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndTimes() {
		ServerCommandGameTime cmd = new ServerCommandGameTime(60000, 30000);
		cmd.setCommandNr(1);
		JsonObject json = cmd.toJsonValue();
		assertEquals("serverGameTime", json.get("netCommandId").asString());
		assertEquals(1, json.get("commandNr").asInt());
		assertEquals(60000L, json.get("gameTime").asLong());
		assertEquals(30000L, json.get("turnTime").asLong());
	}

	@Test
	public void roundTripWithTimes() {
		ServerCommandGameTime cmd = new ServerCommandGameTime(5000, 2000);
		cmd.setCommandNr(8);
		JsonObject json = cmd.toJsonValue();
		ServerCommandGameTime restored = new ServerCommandGameTime().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(8, restored.getCommandNr());
		assertEquals(5000L, restored.getGameTime());
		assertEquals(2000L, restored.getTurnTime());
	}

	@Test
	public void roundTripWithZeroTimes() {
		ServerCommandGameTime cmd = new ServerCommandGameTime();
		JsonObject json = cmd.toJsonValue();
		ServerCommandGameTime restored = new ServerCommandGameTime().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(0L, restored.getGameTime());
		assertEquals(0L, restored.getTurnTime());
	}
}
