package com.fumbbl.ffb.server.step.generator.bb2020;

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
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust push-order tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/generator/bb2020/bomb.rs}.
 */
public class BombFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
	}

	private IStep[] build(String catcherId, boolean passFumble) {
		new Bomb().pushSequence(new Bomb.SequenceParams(gameState, catcherId, passFumble, false));
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: bomb_has_3_steps
	@Test
	public void bombHas3Steps() {
		assertEquals(3, build(null, false).length);
	}

	// Rust: bomb_ends_with_end_bomb_labelled
	@Test
	public void bombEndsWithEndBombLabelled() {
		IStep[] steps = build(null, false);
		IStep last = steps[steps.length - 1];
		assertEquals(StepId.END_BOMB, last.getId());
		assertEquals(IStepLabel.END_BOMB, last.getLabel());
	}

	// Rust: bomb_has_no_resolve_bomb
	@Test
	public void bombHasNoResolveBomb() {
		assertFalse(GeneratorTestSupport.contains(build(null, false), StepId.RESOLVE_BOMB));
	}

	// Rust: bomb_starts_with_init_bomb
	@Test
	public void bombStartsWithInitBomb() {
		assertEquals(StepId.INIT_BOMB, build(null, false)[0].getId());
	}

	// Rust: catcher_id_included_when_some
	@Test
	public void catcherIdIncludedWhenSome() {
		assertEquals("catcher1", GeneratorTestSupport.readField(build("catcher1", false)[0], "fCatcherId"));
	}

	// Rust: pass_fumble_param_passed
	@Test
	public void passFumbleParamPassed() {
		assertTrue(GeneratorTestSupport.booleanField(build(null, true)[0], "fPassFumble"));
	}
}
