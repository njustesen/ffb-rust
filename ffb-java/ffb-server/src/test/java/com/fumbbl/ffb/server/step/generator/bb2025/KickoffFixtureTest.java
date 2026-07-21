package com.fumbbl.ffb.server.step.generator.bb2025;

import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.fixture.GeneratorTestSupport;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.IStepLabel;
import com.fumbbl.ffb.server.step.StepId;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2025/kickoff.rs}.
 */
public class KickoffFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3);
	}

	private IStep[] build(boolean withCoinChoice) {
		new Kickoff().pushSequence(new Kickoff.SequenceParams(gameState, withCoinChoice));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: kickoff_without_coin_choice_starts_with_init_kickoff
	@Test
	public void kickoffWithoutCoinChoiceStartsWithInitKickoff() {
		assertEquals(StepId.INIT_KICKOFF, build(false)[0].getId());
	}

	// Rust: kickoff_with_coin_choice_starts_with_coin_choice
	@Test
	public void kickoffWithCoinChoiceStartsWithCoinChoice() {
		IStep[] steps = build(true);
		assertEquals(StepId.COIN_CHOICE, steps[0].getId());
		assertEquals(StepId.RECEIVE_CHOICE, steps[1].getId());
		assertEquals(StepId.INIT_KICKOFF, steps[2].getId());
	}

	// Rust: kickoff_ends_with_end_kickoff_labelled
	@Test
	public void kickoffEndsWithEndKickoffLabelled() {
		IStep[] steps = build(false);
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.END_KICKOFF, last.getId());
		assertEquals(IStepLabel.END_KICKOFF, last.getLabel());
	}

	// Rust: kickoff_has_two_swarming_steps
	@Test
	public void kickoffHasTwoSwarmingSteps() {
		assertEquals(2, GeneratorTestSupport.count(build(false), StepId.SWARMING));
	}

	// Rust: kickoff_blitz_turn_is_labelled
	@Test
	public void kickoffBlitzTurnIsLabelled() {
		IStep bt = GeneratorTestSupport.find(build(false), StepId.BLITZ_TURN);
		assertNotNull(bt);
		assertEquals(IStepLabel.BLITZ_TURN, bt.getLabel());
	}
}
