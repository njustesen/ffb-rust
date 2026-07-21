package com.fumbbl.ffb.factory;

import com.fumbbl.ffb.FactoryType.Factory;
import com.fumbbl.ffb.inducement.CardType;
import com.fumbbl.ffb.net.NetCommandTestUtil;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/factory/card_type_factory.rs
 * for {@link CardTypeFactory}.
 * <p>
 * The default game context is BB2016, whose card types are exactly {@code magicItem} and
 * {@code dirtyTrick} - matching the two-element collection used by the Rust tests.
 */
public class CardTypeFactoryTest {

	private static CardTypeFactory factory() {
		return NetCommandTestUtil.gameSource().getFactory(Factory.CARD_TYPE);
	}

	@Test
	public void forNameFindsMagicItem() {
		CardType found = factory().forName("magicItem");
		assertEquals("magicItem", found.getName());
	}

	@Test
	public void forNameIsCaseInsensitive() {
		assertNotNull(factory().forName("MAGICITEM"));
	}

	@Test
	public void forNameReturnsNoneForUnknown() {
		assertNull(factory().forName("unknown"));
	}

	@Test
	public void getCardTypesReturnsAll() {
		assertEquals(2, factory().getCardTypes().size());
	}
}
