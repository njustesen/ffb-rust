package com.fumbbl.ffb.server;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/injury_result.rs tests (flag/lifecycle subset).
 *
 * <p>NOTE (test equalization): is_worse_than_* (2) and precedence_known_states are Rust-invented
 * static helpers — Java has no InjuryResult.isWorseThan; injury precedence is inlined inside apply()
 * (Rust-only). The apply_to / handle_ignoring_armour / swap_to_alternate_context / report / bomb-stun
 * tests (~20) drive a live Game + InjuryContext and are being ported via GameFixture in follow-up batches.
 */
public class InjuryResultTest {

	// rust: new_has_correct_defaults
	@Test
	public void newHasCorrectDefaults() {
		InjuryResult r = new InjuryResult();
		assertFalse(r.isAlreadyReported());
		assertTrue(r.isPreRegeneration());
	}

	// rust: passed_regeneration_clears_flag
	@Test
	public void passedRegenerationClearsFlag() {
		InjuryResult r = new InjuryResult();
		r.passedRegeneration();
		assertFalse(r.isPreRegeneration());
	}

	// rust: set_already_reported
	@Test
	public void setAlreadyReported() {
		InjuryResult r = new InjuryResult();
		r.setAlreadyReported(true);
		assertTrue(r.isAlreadyReported());
	}
}
