package com.fumbbl.ffb.kickoff.bb2020;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;

/**
 * Mirror of ffb-rust crates/ffb-model/src/kickoff/bb2020/kickoff_result_mapping.rs tests.
 */
public class KickoffResultMappingTest {

	// rust: roll_2_gives_get_the_ref
	@Test
	public void roll2GivesGetTheRef() {
		assertEquals(KickoffResult.GET_THE_REF, new KickoffResultMapping().getResult(2));
	}

	// rust: roll_7_gives_brilliant_coaching
	@Test
	public void roll7GivesBrilliantCoaching() {
		assertEquals(KickoffResult.BRILLIANT_COACHING, new KickoffResultMapping().getResult(7));
	}

	// rust: roll_12_gives_pitch_invasion
	@Test
	public void roll12GivesPitchInvasion() {
		assertEquals(KickoffResult.PITCH_INVASION, new KickoffResultMapping().getResult(12));
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
