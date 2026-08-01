package com.fumbbl.ffb.server.step.bb2020.block;

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
 * Mirror of ffb-rust crates/ffb-engine/src/step/bb2020/block/step_hit_and_run.rs (guard + param subset).
 * A blitzer without the Hit-and-Run property falls through to NEXT_STEP. END_TURN / END_PLAYER_ACTION
 * accepted via setParameter; unknown → false. The hit-and-run-command coordinate / skill-use + hit-and-run
 * reports / eligible-square-finding tests are command/placement-driven and deferred.
 */
public class StepHitAndRunFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.HIT_AND_RUN);
	}

	// rust: no_hit_and_run_skill_returns_next_step
	@Test
	public void noHitAndRunSkillReturnsNextStep() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// rust: set_parameter_end_turn_accepted
	@Test
	public void setParameterEndTurnAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}

	// rust: set_parameter_end_player_action_accepted
	@Test
	public void setParameterEndPlayerActionAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.END_PLAYER_ACTION, true)));
	}

	// unrecognised parameter returns false
	@Test
	public void unknownParameterReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.ADMIN_MODE, true)));
	}
}
