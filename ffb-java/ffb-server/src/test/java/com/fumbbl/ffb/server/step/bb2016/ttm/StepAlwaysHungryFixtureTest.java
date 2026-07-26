package com.fumbbl.ffb.server.step.bb2016.ttm;

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
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2016/ttm/step_always_hungry.rs} (param + next-step
 * subset). THROWN_PLAYER_ID is stored via setParameter. With the acting player lacking the Always
 * Hungry skill, executeStep takes neither the always-hungry nor the escape branch and returns
 * NEXT_STEP. GOTO_LABEL_ON_SUCCESS/FAILURE are init-consumed (setParameter false, exempt); the
 * always-hungry roll / escape / declined-reroll / report tests need the Always Hungry big-guy skill,
 * dice outcomes, and published-param or report inspection and are deferred.
 */
public class StepAlwaysHungryFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		gameState.getGame().setHomePlaying(true);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.THROW_TEAM_MATE);
		GameFixture.placePlayer(gameState, "away1", 5, 6);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.ALWAYS_HUNGRY);
	}

	// rust: set_parameter_thrown_player_id
	@Test
	public void setParameterThrownPlayerId() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.THROWN_PLAYER_ID, "away1")));
	}

	// rust: set_parameter_unknown_returns_false
	@Test
	public void setParameterUnknownReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}

	// rust: no_always_hungry_skill_returns_next
	@Test
	public void noAlwaysHungrySkillReturnsNext() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.THROWN_PLAYER_ID, "away1"));
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(step));
	}
}
