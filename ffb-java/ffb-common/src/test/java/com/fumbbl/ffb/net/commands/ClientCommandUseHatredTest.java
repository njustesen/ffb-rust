package com.fumbbl.ffb.net.commands;

import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_use_hatred.rs tests.
 * Java field is {@code targetId}, serialised under JSON key {@code playerId}.
 */
public class ClientCommandUseHatredTest {

	@Test
	public void targetStored() {
		ClientCommandUseHatred cmd = new ClientCommandUseHatred("p1");
		assertEquals("p1", cmd.getTargetId());
	}

	@Test
	public void defaultNone() {
		assertNull(new ClientCommandUseHatred().getTargetId());
	}

	@Test
	public void getIdIsClientUseHatred() {
		assertEquals(NetCommandId.CLIENT_USE_HATRED, new ClientCommandUseHatred().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndPlayerId() {
		ClientCommandUseHatred cmd = new ClientCommandUseHatred("p9");
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientUseHatred", json.get("netCommandId").asString());
		assertEquals("p9", json.get("playerId").asString());
	}

	@Test
	public void roundTripWithTargetAndEntropy() {
		ClientCommandUseHatred cmd = new ClientCommandUseHatred("p2");
		cmd.setEntropy((byte) 4);
		JsonObject json = cmd.toJsonValue();
		ClientCommandUseHatred restored = (ClientCommandUseHatred) new ClientCommandUseHatred()
			.initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 4, restored.getEntropy());
		assertEquals("p2", restored.getTargetId());
	}

	@Test
	public void roundTripWithNoTarget() {
		ClientCommandUseHatred cmd = new ClientCommandUseHatred();
		JsonObject json = cmd.toJsonValue();
		ClientCommandUseHatred restored = (ClientCommandUseHatred) new ClientCommandUseHatred()
			.initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getTargetId());
	}
}
