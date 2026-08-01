package com.fumbbl.ffb.server.step;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/step/step_next_step_and_repeat.rs. start() unconditionally
 * returns NEXT_STEP_AND_REPEAT; setParameter always returns false (no keys). The Rust
 * handle_command_returns_next_step_action twin exercises the command path (AbstractStep default, no
 * override on this step) — exempt.
 */
public class StepNextStepAndRepeatFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.NEXT_STEP_AND_REPEAT);
	}

	// rust: start_returns_next_step_and_repeat_action
	@Test
	public void startReturnsNextStepAndRepeatAction() {
		assertEquals(StepAction.NEXT_STEP_AND_REPEAT, GameFixture.startStep(newStep()));
	}

	// rust: set_parameter_returns_false
	@Test
	public void setParameterReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}
}
