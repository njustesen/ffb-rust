package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_punt_to_crowd.rs tests.
 */
public class ClientCommandPuntToCrowdTest {

	@Test
	public void trueStored() {
		ClientCommandPuntToCrowd cmd = new ClientCommandPuntToCrowd(true);
		assertTrue(cmd.isPuntToCrowd());
	}

	@Test
	public void defaultFalse() {
		ClientCommandPuntToCrowd cmd = new ClientCommandPuntToCrowd();
		assertFalse(cmd.isPuntToCrowd());
	}

	@Test
	public void falseStored() {
		ClientCommandPuntToCrowd cmd = new ClientCommandPuntToCrowd(false);
		assertFalse(cmd.isPuntToCrowd());
	}

	@Test
	public void getIdIsClientPuntToCrowd() {
		assertEquals(NetCommandId.CLIENT_PUNT_TO_CROWD, new ClientCommandPuntToCrowd().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndPuntToCrowd() {
		ClientCommandPuntToCrowd cmd = new ClientCommandPuntToCrowd(true);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientPuntToCrowd", json.get("netCommandId").asString());
		assertTrue(json.get("puntToCrowd").asBoolean());
	}

	@Test
	public void roundTripWithData() {
		ClientCommandPuntToCrowd cmd = new ClientCommandPuntToCrowd(true);
		cmd.setEntropy((byte) 6);
		JsonObject json = cmd.toJsonValue();
		ClientCommandPuntToCrowd restored = new ClientCommandPuntToCrowd().initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 6, restored.getEntropy());
		assertTrue(restored.isPuntToCrowd());
	}

	@Test
	public void roundTripDefault() {
		ClientCommandPuntToCrowd cmd = new ClientCommandPuntToCrowd();
		JsonObject json = cmd.toJsonValue();
		ClientCommandPuntToCrowd restored = new ClientCommandPuntToCrowd().initFrom(NetCommandTestUtil.gameSource(), json);
		assertFalse(restored.isPuntToCrowd());
		assertFalse(restored.hasEntropy());
	}
}
