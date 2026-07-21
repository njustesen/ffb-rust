package com.fumbbl.ffb.inducement;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/inducement/inducement_phase.rs for {@link InducementPhase}.
 */
public class InducementPhaseTest {

	@Test
	public void testNameMatchesJava() {
		assertEquals("startOfOwnTurn", InducementPhase.START_OF_OWN_TURN.getName());
	}

	@Test
	public void testDescriptionMatchesJava() {
		assertEquals("before setting up", InducementPhase.BEFORE_SETUP.getDescription());
	}
}
