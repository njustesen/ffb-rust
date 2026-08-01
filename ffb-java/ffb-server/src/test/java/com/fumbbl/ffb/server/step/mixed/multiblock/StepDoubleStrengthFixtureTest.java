package com.fumbbl.ffb.server.step.mixed.multiblock;

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
 * Mirror of ffb-rust crates/ffb-engine/src/step/mixed/multiblock/step_double_strength.rs. With no
 * Dauntless-success targets, or no Indomitable skill on the acting player, start() → NEXT_STEP;
 * PLAYER_ID_DAUNTLESS_SUCCESS adds a target. The with-Indomitable prompt / use-skill publish / report /
 * multi-target-choice tests are command-driven and deferred.
 */
public class StepDoubleStrengthFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.placePlayer(gameState, "away1", 5, 6);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.BLOCK);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.DOUBLE_STRENGTH);
	}

	// rust: no_targets_next_step_immediately
	@Test
	public void noTargetsNextStepImmediately() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// rust: no_indomitable_next_step (target present but acting player lacks Indomitable)
	@Test
	public void noIndomitableNextStep() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.PLAYER_ID_DAUNTLESS_SUCCESS, "away1"));
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(step));
	}

	// rust: PLAYER_ID_DAUNTLESS_SUCCESS accepted via setParameter
	@Test
	public void playerIdDauntlessSuccessParamAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.PLAYER_ID_DAUNTLESS_SUCCESS, "away1")));
	}

	// unrecognised parameter returns false
	@Test
	public void unknownParameterReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}
}
