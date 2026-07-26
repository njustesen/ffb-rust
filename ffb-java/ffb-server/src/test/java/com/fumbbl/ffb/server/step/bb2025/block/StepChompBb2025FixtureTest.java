package com.fumbbl.ffb.server.step.bb2025.block;

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
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2025/block/step_chomp.rs} (param + next-step subset).
 * USING_CHOMP is stored via setParameter. An acting player not using chomp, or using it but lacking
 * the chomp skill, falls through to NEXT_STEP. GOTO_LABEL_ON_END is init-consumed (exempt); the
 * chomp-roll / re-rolled-flag / decline-reroll tests are dice/command driven and deferred.
 */
public class StepChompBb2025FixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		gameState.getGame().setHomePlaying(true);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.BLOCK);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.CHOMP);
	}

	// rust: set_parameter_using_chomp_accepted
	@Test
	public void setParameterUsingChompAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.USING_CHOMP, true)));
	}

	// rust: not_using_chomp_returns_next_step
	@Test
	public void notUsingChompReturnsNextStep() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// rust: using_chomp_but_no_skill_returns_next_step
	@Test
	public void usingChompButNoSkillReturnsNextStep() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.USING_CHOMP, true));
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(step));
	}
}
