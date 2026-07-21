package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_argue_the_call.rs tests.
 */
public class ClientCommandArgueTheCallTest {

	@Test
	public void defaultEmptyPlayerIds() {
		ClientCommandArgueTheCall cmd = new ClientCommandArgueTheCall();
		assertEquals(0, cmd.getPlayerIds().length);
	}

	@Test
	public void withPlayerIdStoresId() {
		ClientCommandArgueTheCall cmd = new ClientCommandArgueTheCall("p1");
		assertArrayEquals(new String[] { "p1" }, cmd.getPlayerIds());
	}

	@Test
	public void withPlayerIdsStoresAll() {
		ClientCommandArgueTheCall cmd = new ClientCommandArgueTheCall(new String[] { "a", "b" });
		assertEquals(2, cmd.getPlayerIds().length);
	}

	@Test
	public void getIdIsClientArgueTheCall() {
		assertEquals(NetCommandId.CLIENT_ARGUE_THE_CALL, new ClientCommandArgueTheCall().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndPlayerIds() {
		ClientCommandArgueTheCall cmd = new ClientCommandArgueTheCall(new String[] { "a", "b" });
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientArgueTheCall", json.get("netCommandId").asString());
		assertEquals(2, json.get("playerIds").asArray().size());
		assertEquals("a", json.get("playerIds").asArray().get(0).asString());
		assertEquals("b", json.get("playerIds").asArray().get(1).asString());
	}

	@Test
	public void roundTripWithPopulatedFields() {
		ClientCommandArgueTheCall cmd = new ClientCommandArgueTheCall(new String[] { "a", "b" });
		cmd.setEntropy((byte) 2);
		JsonObject json = cmd.toJsonValue();
		ClientCommandArgueTheCall restored = new ClientCommandArgueTheCall().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 2, restored.getEntropy());
		assertArrayEquals(new String[] { "a", "b" }, restored.getPlayerIds());
	}

	@Test
	public void roundTripWithDefaultData() {
		ClientCommandArgueTheCall cmd = new ClientCommandArgueTheCall();
		JsonObject json = cmd.toJsonValue();
		ClientCommandArgueTheCall restored = new ClientCommandArgueTheCall().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(0, restored.getPlayerIds().length);
	}
}
