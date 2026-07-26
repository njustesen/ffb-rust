package com.fumbbl.ffb.server.step.bb2016;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.TurnMode;
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
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2016/step_end_feeding.rs} (turn-mode / param subset).
 * The two "publishes EndPlayerAction" tests are deferred — GeneratorTestSupport exposes the pushed
 * sequence but not the published-parameter list, so that assertion has no faithful accessor here.
 */
public class StepEndFeedingFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.END_FEEDING);
	}

	// rust: default_pushes_select_sequence
	@Test
	public void defaultPushesSelectSequence() {
		gameState.getGame().setTurnMode(TurnMode.REGULAR);
		IStep step = newStep();
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(step));
		IStep[] seq = GeneratorTestSupport.sequence(gameState);
		assertTrue(seq.length > 0);
		assertEquals(StepId.INIT_SELECTING, seq[0].getId());
	}

	// rust: end_turn_pass_block_pushes_end_turn_sequence
	@Test
	public void endTurnPassBlockPushesEndTurnSequence() {
		gameState.getGame().setTurnMode(TurnMode.PASS_BLOCK);
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.END_TURN, true));
		GameFixture.startStep(step);
		IStep[] seq = GeneratorTestSupport.sequence(gameState);
		assertTrue(seq.length > 0);
		assertEquals(StepId.END_TURN, seq[0].getId());
	}

	// rust: end_turn_regular_pushes_inducement_end_of_own_turn
	@Test
	public void endTurnRegularPushesInducementEndOfOwnTurn() {
		gameState.getGame().setTurnMode(TurnMode.REGULAR);
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.END_TURN, true));
		GameFixture.startStep(step);
		IStep[] seq = GeneratorTestSupport.sequence(gameState);
		assertTrue(seq.length > 0);
		assertEquals(StepId.INIT_INDUCEMENT, seq[0].getId());
	}

	// rust: default_clears_pass_coordinate
	@Test
	public void defaultClearsPassCoordinate() {
		Game game = gameState.getGame();
		game.setTurnMode(TurnMode.REGULAR);
		game.setPassCoordinate(new FieldCoordinate(5, 5));
		GameFixture.startStep(newStep());
		assertNull(game.getPassCoordinate());
	}

	// rust: set_parameter_end_player_action_accepted
	@Test
	public void setParameterEndPlayerActionAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.END_PLAYER_ACTION, true)));
	}

	// rust: set_parameter_end_turn_accepted
	@Test
	public void setParameterEndTurnAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}

	// rust: unrecognised_parameter_returns_false
	@Test
	public void unrecognisedParameterReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.CHECK_FORGO, true)));
	}
}
