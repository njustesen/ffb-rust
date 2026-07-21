package com.fumbbl.ffb.model;

import com.fumbbl.ffb.net.NetCommandTestUtil;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/blitz_turn_state.rs for {@link BlitzTurnState}.
 */
public class BlitzTurnStateTest {

	@Test
	public void addActivationIncrementsAmountDecrementsAvailable() {
		BlitzTurnState s = new BlitzTurnState(3, 2);
		s.addActivation();
		assertEquals(1, s.getAmount());
		assertEquals(1, s.getAvailable());
	}

	@Test
	public void limitReachedWhenAmountEqualsLimit() {
		BlitzTurnState s = new BlitzTurnState(2, 2);
		s.addActivation();
		assertFalse(s.limitReached());
		s.addActivation();
		assertTrue(s.limitReached());
	}

	@Test
	public void availablePlayersLeftFalseWhenZero() {
		BlitzTurnState s = new BlitzTurnState(1, 1);
		s.addActivation();
		assertFalse(s.availablePlayersLeft());
	}

	@Test
	public void serdeRoundTrip() {
		BlitzTurnState s = new BlitzTurnState(3, 3);
		s.changeActingPlayer();
		BlitzTurnState back = new BlitzTurnState().initFrom(NetCommandTestUtil.applicationSource(), s.toJsonValue());
		assertTrue(back.actingPlayerWasChanged());
		assertEquals(3, back.getLimit());
	}

	@Test
	public void changeActingPlayerIsIdempotent() {
		BlitzTurnState s = new BlitzTurnState(2, 2);
		s.changeActingPlayer();
		s.changeActingPlayer();
		assertTrue(s.actingPlayerWasChanged());
	}

}
