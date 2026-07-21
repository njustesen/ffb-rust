package com.fumbbl.ffb.dialog;

import org.junit.jupiter.api.Test;

import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_penalty_shootout_parameter.rs for
 * {@link DialogPenaltyShootoutParameter}.
 */
public class DialogPenaltyShootoutParameterTest {

	@Test
	public void addShootoutAppendsToAllVecs() {
		DialogPenaltyShootoutParameter p = new DialogPenaltyShootoutParameter();
		p.addShootout(5, 3, true, "Round 1");
		assertEquals(Arrays.asList(5), p.getHomeRolls());
		assertEquals(Arrays.asList(3), p.getAwayRolls());
		assertEquals(Arrays.asList(true), p.getHomeWon());
		assertEquals(Arrays.asList("Round 1"), p.getDescriptions());
	}

	@Test
	public void addShootoutMultipleRoundsAccumulate() {
		DialogPenaltyShootoutParameter p = new DialogPenaltyShootoutParameter();
		p.addShootout(3, 5, false, "Round 1");
		p.addShootout(6, 2, true, "Round 2");
		assertEquals(2, p.getHomeRolls().size());
		assertEquals(5, p.getAwayRolls().get(0));
		assertEquals(true, p.getHomeWon().get(1));
	}

}
