package com.fumbbl.ffb.inducement.bb2016;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/inducement/bb2016/card_handler_key.rs for {@link CardHandlerKey}.
 */
public class CardHandlerKeyTest {

	@Test
	public void allVariantsHaveNames() {
		CardHandlerKey[] variants = new CardHandlerKey[] { CardHandlerKey.CHOP_BLOCK, CardHandlerKey.CUSTARD_PIE,
			CardHandlerKey.DISTRACT, CardHandlerKey.FORCE_SHIELD, CardHandlerKey.ILLEGAL_SUBSTITUTION,
			CardHandlerKey.PIT_TRAP, CardHandlerKey.RABBITS_FOOT, CardHandlerKey.WITCH_BREW };
		for (CardHandlerKey v : variants) {
			assertFalse(v.name().isEmpty());
		}
	}
}
