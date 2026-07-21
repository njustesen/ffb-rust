package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_petty_cash.rs tests.
 */
public class ClientCommandPettyCashTest {

	@Test
	public void newStoresValue() {
		ClientCommandPettyCash cmd = new ClientCommandPettyCash(50_000);
		assertEquals(50_000, cmd.getPettyCash());
	}

	@Test
	public void defaultIsZero() {
		ClientCommandPettyCash cmd = new ClientCommandPettyCash();
		assertEquals(0, cmd.getPettyCash());
	}

	@Test
	public void negativeValueStored() {
		ClientCommandPettyCash cmd = new ClientCommandPettyCash(-1000);
		assertEquals(-1000, cmd.getPettyCash());
	}

	@Test
	public void getIdIsClientPettyCash() {
		assertEquals(NetCommandId.CLIENT_PETTY_CASH, new ClientCommandPettyCash().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndPettyCash() {
		ClientCommandPettyCash cmd = new ClientCommandPettyCash(1234);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientPettyCash", json.get("netCommandId").asString());
		assertEquals(1234, json.get("pettyCash").asInt());
	}

	@Test
	public void roundTripWithData() {
		ClientCommandPettyCash cmd = new ClientCommandPettyCash(-500);
		cmd.setEntropy((byte) 3);
		JsonObject json = cmd.toJsonValue();
		ClientCommandPettyCash restored = new ClientCommandPettyCash().initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 3, restored.getEntropy());
		assertEquals(-500, restored.getPettyCash());
	}

	@Test
	public void roundTripDefault() {
		ClientCommandPettyCash cmd = new ClientCommandPettyCash();
		JsonObject json = cmd.toJsonValue();
		ClientCommandPettyCash restored = new ClientCommandPettyCash().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(0, restored.getPettyCash());
		assertFalse(restored.hasEntropy());
	}
}
