package com.fumbbl.ffb.server.step.bb2020.block;

import com.fumbbl.ffb.BlockResult;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.PlayerState;
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
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2020/block/step_block_choice.rs} (param + result/label
 * subset). BLOCK_RESULT and OLD_DEFENDER_STATE are stored via setParameter. SKULL drops the attacker
 * (FALLING) and continues; BOTH_DOWN gotos the juggernaut label; POW/PUSHBACK goto the pushback label
 * (POW drops the defender, PUSHBACK restores its old state). The report-emitting / tackle-cancels-dodge
 * / watch-out-dodge tests need placed skilled players and report inspection and are deferred; the
 * no-block-result test is exempt (Java dereferences a non-null block result at this step). Goto-label
 * branches assert only the StepAction.GOTO_LABEL (no goto-label-string accessor).
 */
public class StepBlockChoiceBb2020FixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.BLOCK);
		GameFixture.placePlayer(gameState, "away1", 6, 5);
		gameState.getGame().setDefenderId("away1");
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.BLOCK_CHOICE);
	}

	private int stateBase(String playerId) {
		Game game = gameState.getGame();
		return game.getFieldModel().getPlayerState(game.getPlayerById(playerId)).getBase();
	}

	// rust: set_parameter_block_result_accepted
	@Test
	public void setParameterBlockResultAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.BLOCK_RESULT, BlockResult.POW)));
	}

	// rust: set_parameter_old_defender_state_accepted
	@Test
	public void setParameterOldDefenderStateAccepted() {
		assertTrue(newStep().setParameter(
			StepParameter.from(StepParameterKey.OLD_DEFENDER_STATE, new PlayerState(PlayerState.PRONE))));
	}

	// rust: skull_result_sets_attacker_falling_and_next_step
	@Test
	public void skullResultSetsAttackerFallingAndNextStep() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.BLOCK_RESULT, BlockResult.SKULL));
		step.setParameter(StepParameter.from(StepParameterKey.OLD_DEFENDER_STATE, new PlayerState(PlayerState.PRONE)));
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(step));
		assertEquals(PlayerState.FALLING, stateBase("home1"));
	}

	// rust: both_down_gotos_juggernaut_label
	@Test
	public void bothDownGotosJuggernautLabel() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.GOTO_LABEL_ON_JUGGERNAUT, "jug"));
		step.setParameter(StepParameter.from(StepParameterKey.BLOCK_RESULT, BlockResult.BOTH_DOWN));
		assertEquals(StepAction.GOTO_LABEL, GameFixture.startStep(step));
	}

	// rust: pow_gotos_pushback_label
	@Test
	public void powGotosPushbackLabel() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.GOTO_LABEL_ON_PUSHBACK, "push"));
		step.setParameter(StepParameter.from(StepParameterKey.BLOCK_RESULT, BlockResult.POW));
		assertEquals(StepAction.GOTO_LABEL, GameFixture.startStep(step));
		assertEquals(PlayerState.FALLING, stateBase("away1"));
	}

	// rust: pushback_gotos_pushback_label
	@Test
	public void pushbackGotosPushbackLabel() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.GOTO_LABEL_ON_PUSHBACK, "push"));
		step.setParameter(StepParameter.from(StepParameterKey.BLOCK_RESULT, BlockResult.PUSHBACK));
		step.setParameter(StepParameter.from(StepParameterKey.OLD_DEFENDER_STATE, new PlayerState(PlayerState.PRONE)));
		assertEquals(StepAction.GOTO_LABEL, GameFixture.startStep(step));
		assertEquals(PlayerState.PRONE, stateBase("away1"));
	}
}
