package com.fumbbl.ffb.server.step.mixed.shared;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepAction;
import com.fumbbl.ffb.server.step.StepId;
import com.fumbbl.ffb.server.step.StepParameter;
import com.fumbbl.ffb.server.step.StepParameterKey;
import com.fumbbl.ffb.server.step.StepParameterSet;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/step/mixed/shared/step_consume_parameter.rs. start()
 * always returns NEXT_STEP; setParameter consumes (returns true for) exactly the keys registered via
 * the init param CONSUME_PARAMETER (a Collection of StepParameterKey) and defers everything else to
 * super (which returns false). The Rust parameters_to_consume_itself_is_consumed twin is EXEMPT:
 * the Rust threads the consume-set via setParameter(ParametersToConsume), whereas Java consumes
 * CONSUME_PARAMETER through init() (so setParameter(CONSUME_PARAMETER) returns false).
 */
public class StepConsumeParameterFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
	}

	private IStep newStep(StepParameterKey... consume) {
		IStep step = GameFixture.createStep(gameState, StepId.CONSUME_PARAMETER);
		StepParameterSet set = new StepParameterSet();
		if (consume.length > 0) {
			set.add(StepParameter.from(StepParameterKey.CONSUME_PARAMETER, Arrays.asList(consume)));
		}
		step.init(set);
		return step;
	}

	// rust: start_returns_next_step
	@Test
	public void startReturnsNextStep() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// rust: consumes_registered_parameter
	@Test
	public void consumesRegisteredParameter() {
		IStep step = newStep(StepParameterKey.END_TURN);
		assertTrue(step.setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}

	// rust: does_not_consume_unregistered_parameter
	@Test
	public void doesNotConsumeUnregisteredParameter() {
		IStep step = newStep(StepParameterKey.END_TURN);
		assertFalse(step.setParameter(StepParameter.from(StepParameterKey.END_PLAYER_ACTION, true)));
	}

	// rust: multiple_parameter_kinds_registered
	@Test
	public void multipleParameterKindsRegistered() {
		IStep step = newStep(StepParameterKey.END_TURN, StepParameterKey.END_PLAYER_ACTION);
		assertTrue(step.setParameter(StepParameter.from(StepParameterKey.END_TURN, false)));
		assertTrue(step.setParameter(StepParameter.from(StepParameterKey.END_PLAYER_ACTION, false)));
		assertFalse(step.setParameter(StepParameter.from(StepParameterKey.ADMIN_MODE, false)));
	}
}
