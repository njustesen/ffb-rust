package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_hand_over.rs tests.
 */
public class ClientCommandHandOverTest {

	@Test
	public void fieldsStoredCorrectly() {
		ClientCommandHandOver cmd = new ClientCommandHandOver("thrower", "catcher");
		assertEquals("thrower", cmd.getActingPlayerId());
		assertEquals("catcher", cmd.getCatcherId());
	}

	@Test
	public void defaultBothNone() {
		ClientCommandHandOver cmd = new ClientCommandHandOver();
		assertNull(cmd.getActingPlayerId());
		assertNull(cmd.getCatcherId());
	}

	@Test
	public void newConstructorCreatesDefault() {
		ClientCommandHandOver cmd = new ClientCommandHandOver();
		assertNotNull(cmd);
	}

	@Test
	public void catcherIdStored() {
		ClientCommandHandOver cmd = new ClientCommandHandOver("thrower", "catcher2");
		assertEquals("catcher2", cmd.getCatcherId());
	}

	@Test
	public void getIdIsClientHandOver() {
		assertEquals(NetCommandId.CLIENT_HAND_OVER, new ClientCommandHandOver().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndCatcherId() {
		ClientCommandHandOver cmd = new ClientCommandHandOver("thrower", "catcher");
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientHandOver", json.get("netCommandId").asString());
		assertEquals("catcher", json.get("catcherId").asString());
	}

	@Test
	public void roundTripWithPlayersAndEntropy() {
		ClientCommandHandOver cmd = new ClientCommandHandOver("thrower", "catcher");
		cmd.setEntropy((byte) 3);
		JsonObject json = cmd.toJsonValue();
		ClientCommandHandOver restored = new ClientCommandHandOver().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 3, restored.getEntropy());
		assertEquals("thrower", restored.getActingPlayerId());
		assertEquals("catcher", restored.getCatcherId());
	}

	@Test
	public void roundTripDefault() {
		ClientCommandHandOver cmd = new ClientCommandHandOver();
		JsonObject json = cmd.toJsonValue();
		ClientCommandHandOver restored = new ClientCommandHandOver().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getActingPlayerId());
		assertNull(restored.getCatcherId());
	}
}
