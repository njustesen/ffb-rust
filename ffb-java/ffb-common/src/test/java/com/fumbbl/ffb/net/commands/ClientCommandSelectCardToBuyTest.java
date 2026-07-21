package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import com.fumbbl.ffb.net.commands.ClientCommandSelectCardToBuy.Selection;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_select_card_to_buy.rs tests.
 *
 * Java models the two booleans as a {@link Selection} enum
 * (initialDeckChoice, firstCardChoice). Rust's default (false, false) has no
 * matching self-consistent Java default because the Java command's {@code selection}
 * is {@code null} until set; those tests are adapted to assert the null selection.
 */
public class ClientCommandSelectCardToBuyTest {

	@Test
	public void boolsStored() {
		// Rust: new(true, false) -> initialDeckChoice true, firstCardChoice false -> INITIAL_SECOND.
		ClientCommandSelectCardToBuy cmd = new ClientCommandSelectCardToBuy(Selection.INITIAL_SECOND);
		assertTrue(cmd.getSelection().isInitialDeckChoice());
		assertFalse(cmd.getSelection().isFirstCardChoice());
	}

	@Test
	public void defaultIsNull() {
		// Rust default flattens to (false, false); Java leaves selection null.
		ClientCommandSelectCardToBuy cmd = new ClientCommandSelectCardToBuy();
		assertNull(cmd.getSelection());
	}

	@Test
	public void bothTrueStored() {
		ClientCommandSelectCardToBuy cmd = new ClientCommandSelectCardToBuy(Selection.INITIAL_FIRST);
		assertTrue(cmd.getSelection().isInitialDeckChoice());
		assertTrue(cmd.getSelection().isFirstCardChoice());
	}

	@Test
	public void getIdIsClientSelectCardToBuy() {
		assertEquals(NetCommandId.CLIENT_SELECT_CARD_TO_BUY, new ClientCommandSelectCardToBuy(Selection.INITIAL_FIRST).getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndCardSelection() {
		ClientCommandSelectCardToBuy cmd = new ClientCommandSelectCardToBuy(Selection.INITIAL_SECOND);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientSelectCardToBuy", json.get("netCommandId").asString());
		assertEquals("INITIAL_SECOND", json.get("cardSelection").asString());
	}

	@Test
	public void roundTripPopulated() {
		// Rust: new(false, true) -> REROLLED_FIRST.
		ClientCommandSelectCardToBuy cmd = new ClientCommandSelectCardToBuy(Selection.REROLLED_FIRST);
		cmd.setEntropy((byte) 9);
		JsonObject json = cmd.toJsonValue();
		ClientCommandSelectCardToBuy restored = (ClientCommandSelectCardToBuy) new ClientCommandSelectCardToBuy().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(Selection.REROLLED_FIRST, restored.getSelection());
		assertFalse(restored.getSelection().isInitialDeckChoice());
		assertTrue(restored.getSelection().isFirstCardChoice());
		assertEquals((byte) 9, restored.getEntropy());
	}

	@Test
	public void roundTripDefault() {
		ClientCommandSelectCardToBuy cmd = new ClientCommandSelectCardToBuy();
		JsonObject json = cmd.toJsonValue();
		ClientCommandSelectCardToBuy restored = (ClientCommandSelectCardToBuy) new ClientCommandSelectCardToBuy().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getSelection());
		assertFalse(restored.hasEntropy());
	}
}
