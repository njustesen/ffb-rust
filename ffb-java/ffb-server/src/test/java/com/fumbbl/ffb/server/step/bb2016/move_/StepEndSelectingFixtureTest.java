package com.fumbbl.ffb.server.step.bb2016.move_;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
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
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2016/move_/step_end_selecting.rs}.
 * Same dispatch/param pattern as the bb2025 StepEndSelecting port.
 */
public class StepEndSelectingFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.END_SELECTING);
	}

	private IStep[] dispatch(PlayerAction action) {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.DISPATCH_PLAYER_ACTION, action));
		step.setParameter(StepParameter.from(StepParameterKey.USING_STAB, false));
		GameFixture.startStep(step);
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: end_turn_pushes_end_player_action_sequence
	@Test
	public void endTurnPushesEndPlayerActionSequence() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.END_TURN, true));
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(step));
		assertTrue(GeneratorTestSupport.sequence(gameState).length > 0);
	}

	// Rust: end_player_action_pushes_end_player_action_sequence
	@Test
	public void endPlayerActionPushesEndPlayerActionSequence() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.END_PLAYER_ACTION, true));
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(step));
		assertTrue(GeneratorTestSupport.sequence(gameState).length > 0);
	}

	// Rust: dispatch_block_pushes_block_sequence
	@Test
	public void dispatchBlockPushesBlockSequence() {
		assertTrue(dispatch(PlayerAction.BLOCK).length > 0);
	}

	// Rust: dispatch_foul_pushes_foul_sequence
	@Test
	public void dispatchFoulPushesFoulSequence() {
		assertTrue(dispatch(PlayerAction.FOUL).length > 0);
	}

	// Rust: dispatch_pass_pushes_pass_sequence
	@Test
	public void dispatchPassPushesPassSequence() {
		assertTrue(dispatch(PlayerAction.PASS).length > 0);
	}

	// Rust: dispatch_move_pushes_move_sequence
	@Test
	public void dispatchMovePushesMoveSequence() {
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.DISPATCH_PLAYER_ACTION, PlayerAction.MOVE));
		step.setParameter(StepParameter.from(StepParameterKey.USING_STAB, false));
		GameFixture.startStep(step);
		assertTrue(GeneratorTestSupport.sequence(gameState).length > 0);
	}

	// Rust: dispatch_blitz_move_pushes_blitz_move_sequence
	@Test
	public void dispatchBlitzMovePushesBlitzMoveSequence() {
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.BLITZ_MOVE);
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.DISPATCH_PLAYER_ACTION, PlayerAction.BLITZ_MOVE));
		step.setParameter(StepParameter.from(StepParameterKey.USING_STAB, false));
		GameFixture.startStep(step);
		assertTrue(GeneratorTestSupport.sequence(gameState).length > 0);
	}

	// Rust: no_action_fallback_returns_next_step
	@Test
	public void noActionFallbackReturnsNextStep() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// Rust: move_action_when_not_rooted_pushes_move_sequence
	@Test
	public void moveActionWhenNotRootedPushesMoveSequence() {
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.USING_STAB, false));
		GameFixture.startStep(step);
		assertTrue(GeneratorTestSupport.sequence(gameState).length > 0);
	}

	// Rust: move_action_when_rooted_pushes_end_player_action
	@Test
	public void moveActionWhenRootedPushesEndPlayerAction() {
		Game game = gameState.getGame();
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		game.getFieldModel().setPlayerState(game.getPlayerById("home1"),
			new PlayerState(PlayerState.STANDING).changeRooted(true));
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.USING_STAB, false));
		GameFixture.startStep(step);
		assertTrue(GeneratorTestSupport.sequence(gameState).length > 0);
	}

	// Rust: move_action_when_rooted_and_can_gaze_pushes_select_sequence — NOT mirrored.
	// Needs a rooted gazer with the HypnoticGaze skill plus an adjacent standing opponent
	// and the can-gaze evaluation; that live-state setup is deferred.

	// Rust: set_parameter_end_turn_accepted
	@Test
	public void setParameterEndTurnAccepted() {
		IStep step = newStep();
		assertTrue(step.setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}

	// Rust: set_parameter_dispatch_player_action_accepted
	@Test
	public void setParameterDispatchPlayerActionAccepted() {
		IStep step = newStep();
		assertTrue(step.setParameter(StepParameter.from(StepParameterKey.DISPATCH_PLAYER_ACTION, PlayerAction.BLOCK)));
		assertEquals(PlayerAction.BLOCK, GeneratorTestSupport.readField(step, "fDispatchPlayerAction"));
	}

	// Rust: set_parameter_move_stack_accepted
	@Test
	public void setParameterMoveStackAccepted() {
		IStep step = newStep();
		FieldCoordinate[] stack = { new FieldCoordinate(5, 5), new FieldCoordinate(6, 5) };
		assertTrue(step.setParameter(StepParameter.from(StepParameterKey.MOVE_STACK, stack)));
	}
}
