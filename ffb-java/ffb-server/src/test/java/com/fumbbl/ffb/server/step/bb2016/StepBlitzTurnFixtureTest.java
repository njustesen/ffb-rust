package com.fumbbl.ffb.server.step.bb2016;

import com.fumbbl.ffb.TurnMode;
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
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2016/step_blitz_turn.rs} (turn-mode subset).
 * END_TURN is stored via setParameter. First entry (turn mode not BLITZ) switches to BLITZ; a second
 * entry (already BLITZ) with END_TURN switches to KICKOFF, otherwise stays BLITZ; the step always
 * returns NEXT_STEP. The pushes-self-and-select-sequence test inspects the generated sequence and is
 * deferred.
 */
public class StepBlitzTurnFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		gameState.getGame().setHomePlaying(true);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.BLITZ_TURN);
	}

	// rust: set_parameter_end_turn_accepted
	@Test
	public void setParameterEndTurnAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}

	// rust: set_parameter_unknown_returns_false
	@Test
	public void setParameterUnknownReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.KICKING_PLAYER_COORDINATE, null)));
	}

	// rust: first_entry_sets_blitz_mode
	@Test
	public void firstEntrySetsBlitzMode() {
		gameState.getGame().setTurnMode(TurnMode.KICKOFF);
		GameFixture.startStep(newStep());
		assertEquals(TurnMode.BLITZ, gameState.getGame().getTurnMode());
	}

	// rust: second_entry_end_turn_sets_kickoff_mode
	@Test
	public void secondEntryEndTurnSetsKickoffMode() {
		gameState.getGame().setTurnMode(TurnMode.BLITZ);
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.END_TURN, true));
		GameFixture.startStep(step);
		assertEquals(TurnMode.KICKOFF, gameState.getGame().getTurnMode());
	}

	// rust: second_entry_no_end_turn_stays_in_blitz
	@Test
	public void secondEntryNoEndTurnStaysInBlitz() {
		gameState.getGame().setTurnMode(TurnMode.BLITZ);
		GameFixture.startStep(newStep());
		assertEquals(TurnMode.BLITZ, gameState.getGame().getTurnMode());
	}

	// rust: always_returns_next_step
	@Test
	public void alwaysReturnsNextStep() {
		gameState.getGame().setTurnMode(TurnMode.BLITZ);
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}
}
