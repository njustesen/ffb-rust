package com.fumbbl.ffb.server.step.bb2016;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.inducement.InducementPhase;
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
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2016/step_end_inducement.rs}.
 * The inducement phase + end-phase + end-turn flags are all settable parameters, so every branch
 * ports via setParameter + assertion on the pushed sequence's first step (or contains for the
 * two-sequence start-of-own-turn case).
 */
public class StepEndInducementFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.END_INDUCEMENT);
	}

	// rust: no_phase_waits_continue_not_next_step
	@Test
	public void noPhaseWaitsContinueNotNextStep() {
		assertEquals(StepAction.CONTINUE, GameFixture.startStep(newStep()));
		assertEquals(0, GeneratorTestSupport.sequence(gameState).length);
	}

	// rust: end_turn_flag_pushes_end_turn_sequence
	@Test
	public void endTurnFlagPushesEndTurnSequence() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.INDUCEMENT_PHASE, InducementPhase.END_OF_OWN_TURN));
		step.setParameter(StepParameter.from(StepParameterKey.END_TURN, true));
		GameFixture.startStep(step);
		IStep[] seq = GeneratorTestSupport.sequence(gameState);
		assertTrue(seq.length > 0);
		assertEquals(StepId.END_TURN, seq[0].getId());
	}

	// rust: end_of_own_turn_with_end_phase_pushes_end_turn_sequence
	@Test
	public void endOfOwnTurnWithEndPhasePushesEndTurnSequence() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.INDUCEMENT_PHASE, InducementPhase.END_OF_OWN_TURN));
		step.setParameter(StepParameter.from(StepParameterKey.END_INDUCEMENT_PHASE, true));
		GameFixture.startStep(step);
		IStep[] seq = GeneratorTestSupport.sequence(gameState);
		assertTrue(seq.length > 0);
		assertEquals(StepId.END_TURN, seq[0].getId());
	}

	// rust: start_of_own_turn_with_end_phase_pushes_select_and_check_stalling
	@Test
	public void startOfOwnTurnWithEndPhasePushesSelectAndCheckStalling() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.INDUCEMENT_PHASE, InducementPhase.START_OF_OWN_TURN));
		step.setParameter(StepParameter.from(StepParameterKey.END_INDUCEMENT_PHASE, true));
		GameFixture.startStep(step);
		IStep[] seq = GeneratorTestSupport.sequence(gameState);
		assertTrue(GeneratorTestSupport.contains(seq, StepId.INIT_SELECTING));
		assertTrue(GeneratorTestSupport.contains(seq, StepId.CHECK_STALLING));
	}

	// rust: no_end_phase_pushes_inducement_sequence
	@Test
	public void noEndPhasePushesInducementSequence() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.INDUCEMENT_PHASE, InducementPhase.BEFORE_SETUP));
		GameFixture.startStep(step);
		IStep[] seq = GeneratorTestSupport.sequence(gameState);
		assertTrue(seq.length > 0);
		assertEquals(StepId.INIT_INDUCEMENT, seq[0].getId());
	}

	// rust: set_parameter_home_team_accepted
	@Test
	public void setParameterHomeTeamAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.HOME_TEAM, true)));
	}

	// rust: set_parameter_inducement_phase_accepted
	@Test
	public void setParameterInducementPhaseAccepted() {
		assertTrue(newStep().setParameter(
			StepParameter.from(StepParameterKey.INDUCEMENT_PHASE, InducementPhase.BEFORE_SETUP)));
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
