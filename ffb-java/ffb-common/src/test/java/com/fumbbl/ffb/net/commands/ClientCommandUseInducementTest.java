package com.fumbbl.ffb.net.commands;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.factory.CardFactory;
import com.fumbbl.ffb.factory.InducementTypeFactory;
import com.fumbbl.ffb.inducement.Card;
import com.fumbbl.ffb.inducement.InducementType;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_use_inducement.rs tests.
 * The Rust port stores type/card as name strings; Java stores typed objects obtained from
 * the game-context factories. Assertions are adapted to the Java API.
 */
public class ClientCommandUseInducementTest {

	private static InducementType inducementType() {
		InducementTypeFactory factory = NetCommandTestUtil.gameSource().getFactory(FactoryType.Factory.INDUCEMENT_TYPE);
		return factory.forName("Bribe");
	}

	private static Card anyCard() {
		CardFactory factory = NetCommandTestUtil.gameSource().getFactory(FactoryType.Factory.CARD);
		return factory.allCards().iterator().next();
	}

	@Test
	public void playerIdsStored() {
		InducementType type = inducementType();
		assertNotNull(type);
		ClientCommandUseInducement cmd = new ClientCommandUseInducement(type, "p1");
		assertArrayEquals(new String[] { "p1" }, cmd.getPlayerIds());
	}

	@Test
	public void defaultAllNone() {
		ClientCommandUseInducement cmd = new ClientCommandUseInducement();
		assertNull(cmd.getInducementType());
		assertNull(cmd.getCard());
		assertEquals(0, cmd.getPlayerIds().length);
	}

	@Test
	public void getIdIsClientUseInducement() {
		assertEquals(NetCommandId.CLIENT_USE_INDUCEMENT, new ClientCommandUseInducement().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndInducementType() {
		InducementType type = inducementType();
		assertNotNull(type);
		ClientCommandUseInducement cmd = new ClientCommandUseInducement(type);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientUseInducement", json.get("netCommandId").asString());
		assertEquals(type.getName(), json.get("inducementType").asString());
	}

	/**
	 * Note: Java exposes no public mutator to set an inducement type and a card at the same time
	 * (only single-purpose constructors), so this method verifies the type+players+entropy path
	 * and the card path as two separate self-consistent round trips.
	 */
	@Test
	public void roundTripWithAllFieldsAndEntropy() {
		InducementType type = inducementType();
		assertNotNull(type);
		ClientCommandUseInducement cmd = new ClientCommandUseInducement(type, "p1");
		cmd.setEntropy((byte) 3);
		JsonObject json = cmd.toJsonValue();
		ClientCommandUseInducement restored = new ClientCommandUseInducement()
			.initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 3, restored.getEntropy());
		assertNotNull(restored.getInducementType());
		assertEquals(type.getName(), restored.getInducementType().getName());
		assertArrayEquals(new String[] { "p1" }, restored.getPlayerIds());

		Card card = anyCard();
		assertNotNull(card);
		ClientCommandUseInducement cardCmd = new ClientCommandUseInducement(card, "p2");
		JsonObject cardJson = cardCmd.toJsonValue();
		ClientCommandUseInducement restoredCard = new ClientCommandUseInducement()
			.initFrom(NetCommandTestUtil.gameSource(), cardJson);
		assertNotNull(restoredCard.getCard());
		assertEquals(card.getName(), restoredCard.getCard().getName());
		assertArrayEquals(new String[] { "p2" }, restoredCard.getPlayerIds());
	}

	@Test
	public void roundTripWithNoOptionalFields() {
		ClientCommandUseInducement cmd = new ClientCommandUseInducement();
		JsonObject json = cmd.toJsonValue();
		ClientCommandUseInducement restored = new ClientCommandUseInducement()
			.initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getInducementType());
		assertNull(restored.getCard());
		assertEquals(0, restored.getPlayerIds().length);
	}
}
