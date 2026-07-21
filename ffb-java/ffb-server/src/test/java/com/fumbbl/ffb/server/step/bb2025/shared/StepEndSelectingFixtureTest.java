package com.fumbbl.ffb.server.step.bb2025.shared;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerAction;
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
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2025/shared/step_end_selecting.rs}.
 *
 * StepEndSelecting is the last step in a SELECT sequence: it reads accumulated
 * StepParameters and dispatches to the appropriate action sequence by pushing
 * generators onto the step stack. The Rust tests set fields directly and inspect
 * {@code out.pushes}; here we inject the equivalent StepParameters, run start(),
 * and read the pushed sequence off {@code gameState.getStepStack()}.
 */
public class StepEndSelectingFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(0);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.END_SELECTING);
	}

	private IStep[] dispatch(PlayerAction action, StepParameterKey idKey, String idVal) {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.DISPATCH_PLAYER_ACTION, action));
		// Java reads fUsingStab (Boolean tristate) unconditionally in the block/blitz
		// dispatch; Rust's Option<bool> tolerates None. Set it to keep parity.
		step.setParameter(StepParameter.from(StepParameterKey.USING_STAB, false));
		if (idKey != null) {
			step.setParameter(StepParameter.from(idKey, idVal));
		}
		GameFixture.startStep(step);
		return GeneratorTestSupport.sequence(gameState);
	}

	// Rust: start_returns_next
	@Test
	public void startReturnsNext() {
		IStep step = newStep();
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(step));
	}

	// Rust: end_turn_true_pushes_end_player_action_sequence
	@Test
	public void endTurnTruePushesEndPlayerActionSequence() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.END_TURN, true));
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(step));
		IStep[] seq = GeneratorTestSupport.sequence(gameState);
		assertEquals(StepId.REMOVE_TARGET_SELECTION_STATE, seq[0].getId());
	}

	// Rust: end_player_action_true_pushes_sequence
	@Test
	public void endPlayerActionTruePushesSequence() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.END_PLAYER_ACTION, true));
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(step));
		assertTrue(GeneratorTestSupport.sequence(gameState).length > 0);
	}

	// Rust: check_forgo_propagated_to_end_player_action_sequence
	@Test
	public void checkForgoPropagatedToEndPlayerActionSequence() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.END_TURN, true));
		step.setParameter(StepParameter.from(StepParameterKey.CHECK_FORGO, true));
		GameFixture.startStep(step);
		IStep endFeeding = GeneratorTestSupport.find(GeneratorTestSupport.sequence(gameState), StepId.END_FEEDING);
		assertTrue(GeneratorTestSupport.booleanField(endFeeding, "checkForgo"));
	}

	// Rust: dispatch_player_action_block_pushes_block_sequence
	@Test
	public void dispatchBlockPushesBlockSequence() {
		IStep[] seq = dispatch(PlayerAction.BLOCK, StepParameterKey.BLOCK_DEFENDER_ID, "def1");
		assertEquals(StepId.INIT_BLOCKING, seq[0].getId());
	}

	// Rust: dispatch_player_action_move_pushes_move_sequence
	// Java's move dispatch reads the acting player's state, so one must be on the pitch.
	@Test
	public void dispatchMovePushesMoveSequence() {
		gameState = GameFixture.createGameState(3);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.DISPATCH_PLAYER_ACTION, PlayerAction.MOVE));
		step.setParameter(StepParameter.from(StepParameterKey.USING_STAB, false));
		GameFixture.startStep(step);
		assertEquals(StepId.INIT_MOVING, GeneratorTestSupport.sequence(gameState)[0].getId());
	}

	// Rust: dispatch_player_action_foul_pushes_foul_sequence
	@Test
	public void dispatchFoulPushesFoulSequence() {
		assertEquals(StepId.INIT_FOULING,
			dispatch(PlayerAction.FOUL, StepParameterKey.FOUL_DEFENDER_ID, "def2")[0].getId());
	}

	// Rust: dispatch_player_action_blitz_pushes_blitz_block_sequence
	@Test
	public void dispatchBlitzPushesBlitzBlockSequence() {
		assertEquals(StepId.INIT_BLOCKING,
			dispatch(PlayerAction.BLITZ, StepParameterKey.BLOCK_DEFENDER_ID, "def3")[0].getId());
	}

	// Rust: dispatch_player_action_stand_up_pushes_end_player_action
	@Test
	public void dispatchStandUpPushesEndPlayerAction() {
		assertEquals(StepId.REMOVE_TARGET_SELECTION_STATE, dispatch(PlayerAction.STAND_UP, null, null)[0].getId());
	}

	// Rust: dispatch_player_action_stand_up_blitz_sets_blitz_used
	@Test
	public void dispatchStandUpBlitzSetsBlitzUsed() {
		dispatch(PlayerAction.STAND_UP_BLITZ, null, null);
		assertTrue(gameState.getGame().getTurnDataHome().isBlitzUsed());
	}

	// Rust: dispatch_player_action_remove_confusion_sets_has_moved
	@Test
	public void dispatchRemoveConfusionSetsHasMoved() {
		dispatch(PlayerAction.REMOVE_CONFUSION, null, null);
		assertTrue(gameState.getGame().getActingPlayer().hasMoved());
	}

	// Rust: dispatch_player_action_treacherous_pushes_two_sequences
	@Test
	public void dispatchTreacherousPushesSequences() {
		IStep[] seq = dispatch(PlayerAction.TREACHEROUS, null, null);
		assertTrue(GeneratorTestSupport.contains(seq, StepId.TREACHEROUS));
	}

	// Rust: dispatch_player_action_blitz_select_pushes_select_blitz_target
	@Test
	public void dispatchBlitzSelectPushesSelectBlitzTarget() {
		assertEquals(StepId.SELECT_BLITZ_TARGET, dispatch(PlayerAction.BLITZ_SELECT, null, null)[0].getId());
	}

	// Rust: dispatch_player_action_blitz_move_pushes_blitz_move_sequence
	@Test
	public void dispatchBlitzMovePushesBlitzMoveSequence() {
		assertEquals(StepId.INIT_MOVING, dispatch(PlayerAction.BLITZ_MOVE, null, null)[0].getId());
	}

	// Rust: dispatch_player_action_punt_pushes_punt_sequence
	@Test
	public void dispatchPuntPushesPuntSequence() {
		assertEquals(StepId.INIT_PUNT, dispatch(PlayerAction.PUNT, null, null)[0].getId());
	}

	// Rust: dispatch_player_action_furious_outburst_pushes_sequence
	@Test
	public void dispatchFuriousOutburstPushesSequence() {
		assertEquals(StepId.INIT_FURIOUS_OUTBURST, dispatch(PlayerAction.FURIOUS_OUTPBURST, null, null)[0].getId());
	}

	// Rust: dispatch_player_action_throw_keg_pushes_throw_keg_sequence
	@Test
	public void dispatchThrowKegPushesThrowKegSequence() {
		IStep[] seq = dispatch(PlayerAction.THROW_KEG, StepParameterKey.TARGET_PLAYER_ID, "p99");
		assertEquals(StepId.INIT_ACTIVATION, seq[0].getId());
		assertTrue(GeneratorTestSupport.contains(seq, StepId.THROW_KEG));
	}

	// Rust: set_parameter_block_defender_id
	@Test
	public void setParameterBlockDefenderId() {
		IStep step = newStep();
		assertTrue(step.setParameter(StepParameter.from(StepParameterKey.BLOCK_DEFENDER_ID, "def1")));
		assertEquals("def1", GeneratorTestSupport.readField(step, "fBlockDefenderId"));
	}

	// Rust: set_parameter_move_stack
	@Test
	public void setParameterMoveStack() {
		IStep step = newStep();
		FieldCoordinate[] coords = { new FieldCoordinate(5, 5), new FieldCoordinate(6, 5) };
		assertTrue(step.setParameter(StepParameter.from(StepParameterKey.MOVE_STACK, coords)));
		assertEquals(2, ((FieldCoordinate[]) GeneratorTestSupport.readField(step, "fMoveStack")).length);
	}

	// Rust: set_parameter_check_forgo
	@Test
	public void setParameterCheckForgo() {
		IStep step = newStep();
		assertTrue(step.setParameter(StepParameter.from(StepParameterKey.CHECK_FORGO, true)));
		assertTrue(GeneratorTestSupport.booleanField(step, "checkForgo"));
	}

	// Rust: set_parameter_dispatch_player_action
	@Test
	public void setParameterDispatchPlayerAction() {
		IStep step = newStep();
		assertTrue(step.setParameter(StepParameter.from(StepParameterKey.DISPATCH_PLAYER_ACTION, PlayerAction.BLOCK)));
		assertEquals(PlayerAction.BLOCK, GeneratorTestSupport.readField(step, "fDispatchPlayerAction"));
	}
}
