package com.fumbbl.ffb;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-model/src/inducement/card_target.rs tests.
 */
public class CardTargetTest {

	@Test
	public void fromIdRoundTrips() {
		for (CardTarget t : new CardTarget[] { CardTarget.TURN, CardTarget.OWN_PLAYER, CardTarget.OPPOSING_PLAYER,
			CardTarget.ANY_PLAYER }) {
			assertEquals(t, CardTarget.fromId(t.getId()));
		}
		assertNull(CardTarget.fromId(99));
	}

	@Test
	public void fromNameRoundTripsCaseInsensitively() {
		assertEquals(CardTarget.OWN_PLAYER, CardTarget.fromName("ownPlayer"));
		assertEquals(CardTarget.OWN_PLAYER, CardTarget.fromName("OWNPLAYER"));
		assertEquals(CardTarget.OPPOSING_PLAYER, CardTarget.fromName("opposingPlayer"));
		assertNull(CardTarget.fromName("no-such-target"));
	}
}
