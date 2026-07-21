package com.fumbbl.ffb.inducement.bb2016;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertNotNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/inducement/bb2016/card_type.rs for {@link CardType}.
 * The Rust variant exposes String ids; in Java the ids are {@code GameOptionId} enum values, so "non-empty"
 * maps to "non-null".
 */
public class CardTypeTest {

	@Test
	public void bothVariantsHaveNonEmptyIds() {
		for (CardType ct : new CardType[] { CardType.MAGIC_ITEM, CardType.DIRTY_TRICK }) {
			assertNotNull(ct.getMaxId());
			assertNotNull(ct.getCostId());
		}
	}
}
