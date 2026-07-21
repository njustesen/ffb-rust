package com.fumbbl.ffb.inducement.bb2016;

import com.fumbbl.ffb.inducement.Card;
import com.fumbbl.ffb.inducement.InducementDuration;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/inducement/bb2016/cards.rs for {@link Cards}.
 */
public class CardsTest {

	private static Card find(Cards cards, String name) {
		return cards.allCards().stream().filter(card -> card.getName().equals(name)).findFirst().orElseThrow(AssertionError::new);
	}

	@Test
	public void has24Cards() {
		Cards c = new Cards();
		assertEquals(24, c.allCards().size());
	}

	@Test
	public void allCardsHaveADuration() {
		Cards c = new Cards();
		for (Card card : c.allCards()) {
			assertNotNull(card.getDuration(), card.getName() + " is missing a duration");
		}
	}

	@Test
	public void chopBlockRequiresBlockablePlayerSelection() {
		Cards c = new Cards();
		assertTrue(find(c, "Chop Block").requiresBlockablePlayerSelection());
		for (Card card : c.allCards()) {
			if (!card.getName().equals("Chop Block")) {
				assertFalse(card.requiresBlockablePlayerSelection(),
					card.getName() + " should not require blockable player selection");
			}
		}
	}

	@Test
	public void durationCountsMatchJavaSource() {
		Cards c = new Cards();
		assertEquals(7, count(c, InducementDuration.UNTIL_END_OF_GAME));
		assertEquals(6, count(c, InducementDuration.UNTIL_END_OF_DRIVE));
		assertEquals(7, count(c, InducementDuration.UNTIL_END_OF_TURN));
		assertEquals(1, count(c, InducementDuration.WHILE_HOLDING_THE_BALL));
		assertEquals(1, count(c, InducementDuration.UNTIL_USED));
		assertEquals(2, count(c, InducementDuration.UNTIL_END_OF_OPPONENTS_TURN));
	}

	private static long count(Cards c, InducementDuration duration) {
		return c.allCards().stream().filter(card -> card.getDuration() == duration).count();
	}
}
