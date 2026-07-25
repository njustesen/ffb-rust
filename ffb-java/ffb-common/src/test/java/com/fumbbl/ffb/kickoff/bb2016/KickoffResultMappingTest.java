package com.fumbbl.ffb.kickoff.bb2016;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-model/src/kickoff/bb2016/kickoff_result_mapping.rs tests.
 */
public class KickoffResultMappingTest {

	// rust: roll_2_gives_get_the_ref
	@Test
	public void roll2GivesGetTheRef() {
		assertEquals(KickoffResult.GET_THE_REF, new KickoffResultMapping().getResult(2));
	}

	// rust: roll_3_gives_riot
	@Test
	public void roll3GivesRiot() {
		assertEquals(KickoffResult.RIOT, new KickoffResultMapping().getResult(3));
	}

	// rust: roll_12_gives_pitch_invasion
	@Test
	public void roll12GivesPitchInvasion() {
		assertEquals(KickoffResult.PITCH_INVASION, new KickoffResultMapping().getResult(12));
	}

	// rust: invalid_roll_gives_none
	@Test
	public void invalidRollGivesNone() {
		assertNull(new KickoffResultMapping().getResult(1));
	}

	// rust: all_11_rolls_present
	@Test
	public void all11RollsPresent() {
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
