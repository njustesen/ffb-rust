package com.fumbbl.ffb.factory;

import com.fumbbl.ffb.FactoryType.Factory;
import com.fumbbl.ffb.inducement.Card;
import com.fumbbl.ffb.net.NetCommandTestUtil;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/factory/card_factory.rs
 * for {@link CardFactory}.
 * <p>
 * The default game context is BB2016, matching the Rust tests which use the BB2016 card set.
 */
public class CardFactoryTest {

	private static CardFactory factory() {
		return NetCommandTestUtil.gameSource().getFactory(Factory.CARD);
	}

	@Test
	public void forNameFindsCardCaseInsensitively() {
		Card card = factory().forName("force shield");
		assertEquals("Force Shield", card.getName());
	}

	@Test
	public void forNameReturnsNoneForUnknown() {
		assertNull(factory().forName("Nonexistent Card"));
	}

	@Test
	public void forNameFindsCardWithoutHandlerKey() {
		Card card = factory().forName("Beguiling Bracers");
		assertFalse(card.handlerKey().isPresent());
	}
}
