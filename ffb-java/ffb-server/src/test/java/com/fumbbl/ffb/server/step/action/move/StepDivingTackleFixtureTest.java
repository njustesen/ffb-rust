package com.fumbbl.ffb.server.step.action.move;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepAction;
import com.fumbbl.ffb.server.step.StepId;
import com.fumbbl.ffb.server.step.StepParameter;
import com.fumbbl.ffb.server.step.StepParameterKey;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/action/move/step_diving_tackle.rs} (param subset + the
 * no-COORDINATE_FROM guard). The eligible-tackler prompt / dodge-roll / goto-label / command tests are
 * dice/command-driven and deferred. COORDINATE_FROM/TO, DODGE_ROLL, USING_BREAK_TACKLE/
 * USING_MODIFYING_SKILL/USING_DIVING_TACKLE are accepted via setParameter; unrecognised keys return
 * false; without COORDINATE_FROM the step falls straight through to NEXT_STEP.
 */
public class StepDivingTackleFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.DIVING_TACKLE);
	}

	// rust: parameters_stored_correctly
	@Test
	public void parametersStoredCorrectly() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.COORDINATE_FROM, new FieldCoordinate(5, 5))));
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.COORDINATE_TO, new FieldCoordinate(6, 5))));
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.DODGE_ROLL, 3)));
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.USING_BREAK_TACKLE, true)));
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.USING_MODIFYING_SKILL, true)));
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.USING_DIVING_TACKLE, true)));
	}

	// rust: unrecognised_parameter_returns_false
	@Test
	public void unrecognisedParameterReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}

	// rust: no_coordinate_from_returns_next_step
	@Test
	public void noCoordinateFromReturnsNextStep() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}
}
