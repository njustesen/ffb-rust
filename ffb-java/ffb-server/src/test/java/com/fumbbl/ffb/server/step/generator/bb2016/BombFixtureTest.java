package com.fumbbl.ffb.server.step.generator.bb2016;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.fixture.GeneratorTestSupport;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.IStepLabel;
import com.fumbbl.ffb.server.step.StepId;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2016/bomb.rs}.
 *
 * Rust's {@code allow_move_after_pass_passed_to_end_bomb} is not mirrored: the Java
 * StepEndBomb does not retain ALLOW_MOVE_AFTER_PASS in a readable field.
 */
public class BombFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep[] build(String catcherId) {
		new Bomb().pushSequence(new Bomb.SequenceParams(gameState, catcherId, false, false, false));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: bomb_starts_with_init_bomb
	@Test
	public void bombStartsWithInitBomb() {
		assertEquals(StepId.INIT_BOMB, build(null)[0].getId());
	}

	// Rust: bomb_ends_with_end_bomb_labelled
	@Test
	public void bombEndsWithEndBombLabelled() {
		IStep[] steps = build(null);
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.END_BOMB, last.getId());
		assertEquals(IStepLabel.END_BOMB, last.getLabel());
	}

	// Rust: bomb_has_catch_scatter_throw_in
	@Test
	public void bombHasCatchScatterThrowIn() {
		assertTrue(GeneratorTestSupport.contains(build(null), StepId.CATCH_SCATTER_THROW_IN));
	}

	// Rust: bomb_sequence_has_exactly_three_steps
	@Test
	public void bombSequenceHasExactlyThreeSteps() {
		assertEquals(3, build(null).length);
	}

	// Rust: catcher_id_included_when_some
	@Test
	public void catcherIdIncludedWhenSome() {
		assertEquals("catcher1", GeneratorTestSupport.readField(build("catcher1")[0], "fCatcherId"));
	}
}
