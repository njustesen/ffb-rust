package com.fumbbl.ffb.server.step.bb2025.special;

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
 * Mirror of ffb-rust crates/ffb-engine/src/step/bb2025/special/step_recheck_explode_skill.rs (guard +
 * param subset). With no catcher (and/or the bomb-bounces option off, and/or skip set), the step marks
 * the bomb used and passes straight through to NEXT_STEP. SKIP / CATCHER_ID accepted via setParameter;
 * unknown → false. The explode-skill prompt / KABOOM use-skill-command / clear-catcher tests are
 * command + game-option-driven and deferred.
 */
public class StepRecheckExplodeSkillFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.RECHECK_EXPLODE_SKILL);
	}

	// rust: default passes straight through (no catcher → guard fails → NEXT_STEP)
	@Test
	public void defaultPassesStraightThrough() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// rust: set_parameter_skip_and_catcher_id
	@Test
	public void setParameterSkipAndCatcherId() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.SKIP, true)));
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.CATCHER_ID, "away1")));
	}

	// rust: set_parameter_unrelated_returns_false
	@Test
	public void setParameterUnrelatedReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}
}
