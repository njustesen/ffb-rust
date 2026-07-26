package com.fumbbl.ffb.server.step.bb2016.block;

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
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2016/block/step_end_blocking.rs} (dispatch/param subset).
 * The use-skill ADD_BLOCK_DIE report tests and the held-in-place force-second-block regression test
 * (Frenzy attacker + Tentacles + adjacent defender) are deferred to a follow-up.
 */
public class StepEndBlockingFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.END_BLOCKING);
	}

	// rust: default_state_pushes_end_player_action_sequence
	// Java's execute path dereferences the acting player (StepEndBlocking always runs at the end of a
	// block sequence, so one is present in production) — place a standing BLOCK acting player.
	@Test
	public void defaultStatePushesEndPlayerActionSequence() {
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", com.fumbbl.ffb.PlayerAction.BLOCK);
		IStep step = newStep();
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(step));
		IStep[] sequence = GeneratorTestSupport.sequence(gameState);
		assertTrue(sequence.length > 0);
		assertEquals(StepId.INIT_FEEDING, sequence[0].getId());
	}

	// rust: end_turn_clears_defender_and_pushes_sequence
	@Test
	public void endTurnClearsDefenderAndPushesSequence() {
		Game game = gameState.getGame();
		game.setDefenderId("def1");
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.END_TURN, true));
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(step));
		assertNull(game.getDefenderId());
		assertTrue(GeneratorTestSupport.sequence(gameState).length > 0);
	}

	// rust: end_player_action_clears_defender
	@Test
	public void endPlayerActionClearsDefender() {
		Game game = gameState.getGame();
		game.setDefenderId("def1");
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.END_PLAYER_ACTION, true));
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(step));
		assertNull(game.getDefenderId());
	}

	// rust: set_parameter_all_flags_accepted
	@Test
	public void setParameterAllFlagsAccepted() {
		IStep step = newStep();
		assertTrue(step.setParameter(StepParameter.from(StepParameterKey.DEFENDER_PUSHED, true)));
		assertTrue(step.setParameter(StepParameter.from(StepParameterKey.END_PLAYER_ACTION, true)));
		assertTrue(step.setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
		assertTrue(step.setParameter(StepParameter.from(StepParameterKey.USING_STAB, true)));
	}

	// rust: set_parameter_old_defender_state
	@Test
	public void setParameterOldDefenderState() {
		IStep step = newStep();
		assertTrue(step.setParameter(
			StepParameter.from(StepParameterKey.OLD_DEFENDER_STATE, new PlayerState(PlayerState.STANDING))));
	}

	// rust: unrecognised_parameter_returns_false
	@Test
	public void unrecognisedParameterReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.GOTO_LABEL, "x")));
	}
}
