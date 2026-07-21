package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_use_brawler.rs tests.
 */
public class ClientCommandUseBrawlerTest {

	@Test
	public void defaultHasNoTargetId() {
		ClientCommandUseBrawler cmd = new ClientCommandUseBrawler();
		assertNull(cmd.getTargetId());
	}

	@Test
	public void withTargetIdStoresValue() {
		ClientCommandUseBrawler cmd = new ClientCommandUseBrawler("t-1");
		assertEquals("t-1", cmd.getTargetId());
	}

	@Test
	public void getIdIsClientUseBrawler() {
		assertEquals(NetCommandId.CLIENT_USE_BRAWLER, new ClientCommandUseBrawler().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndPlayerIdKey() {
		ClientCommandUseBrawler cmd = new ClientCommandUseBrawler("t-1");
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientUseBrawler", json.get("netCommandId").asString());
		assertEquals("t-1", json.get("playerId").asString());
	}

	@Test
	public void roundTripWithTargetIdAndEntropy() {
		ClientCommandUseBrawler cmd = new ClientCommandUseBrawler("t-2");
		cmd.setEntropy((byte) 4);
		JsonObject json = cmd.toJsonValue();
		ClientCommandUseBrawler restored = (ClientCommandUseBrawler)
			new ClientCommandUseBrawler().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 4, restored.getEntropy());
		assertEquals("t-2", restored.getTargetId());
	}

	@Test
	public void roundTripWithNoTargetId() {
		ClientCommandUseBrawler cmd = new ClientCommandUseBrawler();
		JsonObject json = cmd.toJsonValue();
		ClientCommandUseBrawler restored = (ClientCommandUseBrawler)
			new ClientCommandUseBrawler().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getTargetId());
	}
}
