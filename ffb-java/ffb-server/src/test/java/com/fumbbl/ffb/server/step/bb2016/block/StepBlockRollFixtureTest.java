package com.fumbbl.ffb.server.step.bb2016.block;

import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
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
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2016/block/step_block_roll.rs} (param + no-result subset).
 * With no block result yet, the step rolls the block dice and shows the block-roll dialog, so start
 * returns CONTINUE. The publishes-NR_OF_DICE/BLOCK_RESULT, block-choice command, report-emission, and
 * negative-nr-of-dice dialog-team-swap tests inspect published params / reports / dialogs and are
 * deferred (the swap is a Rust bug fixed there; Java is ground truth for it).
 */
public class StepBlockRollFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.placePlayer(gameState, "away1", 6, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.BLOCK);
		Game game = gameState.getGame();
		game.setDefenderId("away1");
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.BLOCK_ROLL);
	}

	// rust: successful_dauntless_parameter_accepted
	@Test
	public void successfulDauntlessParameterAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.SUCCESSFUL_DAUNTLESS, true)));
	}

	// rust: start_with_no_result_stays_cont
	@Test
	public void startWithNoResultStaysContinue() {
		assertEquals(StepAction.CONTINUE, GameFixture.startStep(newStep()));
	}
}
