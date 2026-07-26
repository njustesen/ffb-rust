package com.fumbbl.ffb.server.step.bb2016.move_;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.fixture.GeneratorTestSupport;
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
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2016/move_/step_end_moving.rs} (dispatch/param subset).
 * The acting-player/skill/mate-dependent tests (block/foul-not-moving, hand-over, can-gaze,
 * kick/throw-team-mate) are deferred to a follow-up with the placement fixture.
 */
public class StepEndMovingFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.END_MOVING);
	}

	// rust: start_returns_next_step
	@Test
	public void startReturnsNextStep() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// rust: end_turn_pushes_end_player_action_sequence
	@Test
	public void endTurnPushesEndPlayerActionSequence() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.END_TURN, true));
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(step));
		assertTrue(GeneratorTestSupport.sequence(gameState).length > 0);
	}

	// rust: end_player_action_pushes_end_player_action_sequence
	@Test
	public void endPlayerActionPushesEndPlayerActionSequence() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.END_PLAYER_ACTION, true));
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(step));
		assertTrue(GeneratorTestSupport.sequence(gameState).length > 0);
	}

	// rust: block_defender_id_pushes_block_sequence
	@Test
	public void blockDefenderIdPushesBlockSequence() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.BLOCK_DEFENDER_ID, "def1"));
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(step));
		assertTrue(GeneratorTestSupport.sequence(gameState).length > 0);
	}

	// rust: move_stack_pushes_move_sequence
	@Test
	public void moveStackPushesMoveSequence() {
		IStep step = newStep();
		FieldCoordinate[] stack = { new FieldCoordinate(5, 5) };
		step.setParameter(StepParameter.from(StepParameterKey.MOVE_STACK, stack));
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(step));
		assertTrue(GeneratorTestSupport.sequence(gameState).length > 0);
	}

	// rust: set_parameter_end_turn_accepted
	@Test
	public void setParameterEndTurnAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}

	// rust: set_parameter_move_stack_accepted
	@Test
	public void setParameterMoveStackAccepted() {
		FieldCoordinate[] stack = { new FieldCoordinate(5, 5) };
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.MOVE_STACK, stack)));
	}

	// rust: set_parameter_block_defender_id_accepted
	@Test
	public void setParameterBlockDefenderIdAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.BLOCK_DEFENDER_ID, "d1")));
	}

	// rust: set_parameter_feeding_allowed_accepted
	@Test
	public void setParameterFeedingAllowedAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.FEEDING_ALLOWED, false)));
	}

	// rust: unrecognised_parameter_returns_false
	@Test
	public void unrecognisedParameterReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.DODGE_ROLL, 3)));
	}
}
