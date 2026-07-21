package com.fumbbl.ffb.inducement.bb2020;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/inducement/bb2020/cards.rs for {@link Cards}.
 */
public class CardsTest {

	@Test
	public void bb2020CardsIsEmpty() {
		Cards c = new Cards();
		assertEquals(0, c.allCards().size());
	}
}
