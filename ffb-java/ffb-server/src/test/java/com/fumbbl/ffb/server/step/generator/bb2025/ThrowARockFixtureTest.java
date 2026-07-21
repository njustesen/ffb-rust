package com.fumbbl.ffb.server.step.generator.bb2025;

import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.fixture.GeneratorTestSupport;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepId;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2025/throw_a_rock.rs}.
 */
public class ThrowARockFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3);
	}

	private IStep[] build(boolean homeTeam) {
		new ThrowARock().pushSequence(new ThrowARock.SequenceParams(gameState, homeTeam));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: throw_a_rock_has_6_steps
	@Test
	public void throwARockHas6Steps() {
		assertEquals(6, build(false).length);
	}

	// Rust: throw_a_rock_starts_with_throw_a_rock
	@Test
	public void throwARockStartsWithThrowARock() {
		assertEquals(StepId.THROW_A_ROCK, build(false)[0].getId());
	}

	// Rust: home_team_param_passed_to_first_step
	@Test
	public void homeTeamParamPassedToFirstStep() {
		assertTrue(GeneratorTestSupport.booleanField(build(true)[0], "homeTeam"));
	}

	// Rust: away_team_param_passed_to_first_step
	@Test
	public void awayTeamParamPassedToFirstStep() {
		assertFalse(GeneratorTestSupport.booleanField(build(false)[0], "homeTeam"));
	}

	// Rust: contains_catch_scatter_throw_in_as_last
	@Test
	public void containsCatchScatterThrowInAsLast() {
		IStep[] steps = build(false);
		assertEquals(StepId.CATCH_SCATTER_THROW_IN, steps[steps.length - 1].getId());
	}
}
