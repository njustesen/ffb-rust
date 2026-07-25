package com.fumbbl.ffb.kickoff.bb2025;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-model/src/kickoff/bb2025/kickoff_result_mapping.rs tests.
 */
public class KickoffResultMappingTest {

	// rust: roll_10_gives_charge
	@Test
	public void roll10GivesCharge() {
		assertEquals(KickoffResult.CHARGE, new KickoffResultMapping().getResult(10));
	}

	// rust: roll_11_gives_dodgy_snack
	@Test
	public void roll11GivesDodgySnack() {
		assertEquals(KickoffResult.DODGY_SNACK, new KickoffResultMapping().getResult(11));
	}

	// rust: roll_7_gives_brilliant_coaching
	@Test
	public void roll7GivesBrilliantCoaching() {
		assertEquals(KickoffResult.BRILLIANT_COACHING, new KickoffResultMapping().getResult(7));
	}

	// rust: all_11_present
	@Test
	public void all11Present() {
		KickoffResultMapping m = new KickoffResultMapping();
		int count = 0;
		for (int r = 2; r <= 12; r++) {
			if (m.getResult(r) != null) {
				count++;
			}
		}
		assertEquals(11, count);
	}
}
