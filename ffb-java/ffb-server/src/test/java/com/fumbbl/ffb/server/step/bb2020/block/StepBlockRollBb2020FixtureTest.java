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
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2020/block/step_block_roll.rs} (param + no-result
 * subset). SUCCESSFUL_DAUNTLESS and DOUBLE_TARGET_STRENGTH are stored via setParameter. With no block
 * result yet, the step rolls the block dice and shows the block-roll dialog -> CONTINUE. The dice-count
 * / published-param / block-choice command / additional-assists / skill-reroll (brawler, hatred, pro,
 * savage blow, TRR) / report tests inspect dice, published params, commands, or reports and are
 * deferred.
 */
public class StepBlockRollBb2020FixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		gameState.getGame().setHomePlaying(true);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.placePlayer(gameState, "away1", 6, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.BLOCK);
		gameState.getGame().setDefenderId("away1");
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.BLOCK_ROLL);
	}

	// rust: successful_dauntless_parameter_accepted
	@Test
	public void successfulDauntlessParameterAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.SUCCESSFUL_DAUNTLESS, true)));
	}

	// rust: double_target_strength_parameter_accepted
	@Test
	public void doubleTargetStrengthParameterAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.DOUBLE_TARGET_STRENGTH, true)));
	}

	// rust: start_with_no_result_stays_cont
	@Test
	public void startWithNoResultStaysContinue() {
		assertEquals(StepAction.CONTINUE, GameFixture.startStep(newStep()));
	}
}
