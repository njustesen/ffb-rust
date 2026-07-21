package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.inducement.CardType;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_buy_card.rs tests.
 *
 * Discrepancy: the Rust struct stores an arbitrary card-type name string ("BRIBE",
 * "BLOODWEISER_BABES"). Java's ClientCommandBuyCard stores a real CardType whose wire form is
 * CardType.getName() (e.g. "dirtyTrick", "magicItem") resolved via the CardType factory. These
 * tests therefore use real CardType values from the bb2016 CardType enum; the asserted wire
 * strings differ from the Rust ones accordingly.
 */
public class ClientCommandBuyCardTest {

	@Test
	public void defaultHasNoCardType() {
		ClientCommandBuyCard cmd = new ClientCommandBuyCard();
		assertNull(cmd.getCardType());
	}

	@Test
	public void storesCardTypeName() {
		CardType cardType = com.fumbbl.ffb.inducement.bb2016.CardType.DIRTY_TRICK;
		ClientCommandBuyCard cmd = new ClientCommandBuyCard(cardType);
		assertEquals(cardType, cmd.getCardType());
	}

	@Test
	public void getIdIsClientBuyCard() {
		assertEquals(NetCommandId.CLIENT_BUY_CARD, new ClientCommandBuyCard().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndCardType() {
		CardType cardType = com.fumbbl.ffb.inducement.bb2016.CardType.DIRTY_TRICK;
		ClientCommandBuyCard cmd = new ClientCommandBuyCard(cardType);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientBuyCard", json.get("netCommandId").asString());
		assertEquals(cardType.getName(), json.get("cardType").asString());
	}

	@Test
	public void roundTripWithCardTypeAndEntropy() {
		CardType cardType = com.fumbbl.ffb.inducement.bb2016.CardType.MAGIC_ITEM;
		ClientCommandBuyCard cmd = new ClientCommandBuyCard(cardType);
		cmd.setEntropy((byte) 4);
		JsonObject json = cmd.toJsonValue();
		ClientCommandBuyCard restored = new ClientCommandBuyCard().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 4, restored.getEntropy());
		assertEquals(cardType, restored.getCardType());
	}

	@Test
	public void roundTripWithNoCardType() {
		ClientCommandBuyCard cmd = new ClientCommandBuyCard();
		JsonObject json = cmd.toJsonValue();
		ClientCommandBuyCard restored = new ClientCommandBuyCard().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getCardType());
	}
}
