package com.fumbbl.ffb.server.step.bb2016.block;

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
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2016/block/step_block_choice.rs} (result/label subset).
 * The report-emitting tests (POW/SKULL ReportBlockChoice, tackle-cancels-dodge, right-stuff, dodge
 * label branches) need placed skilled players + report inspection and are deferred. The goto-label
 * branches are asserted via StepAction.GOTO_LABEL + the resulting player state (GeneratorTestSupport
 * exposes no goto-label-string accessor). The Rust no_block_result_returns_next test is EXEMPT —
 * Java's StepBlockChoice dereferences a non-null fBlockResult at this step (always set in production).
 */
public class StepBlockChoiceFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
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

	// rust: skull_attacker_falls_defender_restores
	@Test
	public void skullAttackerFallsDefenderRestores() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.BLOCK_RESULT, BlockResult.SKULL));
		step.setParameter(StepParameter.from(StepParameterKey.OLD_DEFENDER_STATE, new PlayerState(PlayerState.PRONE)));
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(step));
		assertEquals(PlayerState.FALLING, stateBase("home1"));
		assertEquals(PlayerState.PRONE, stateBase("away1"));
	}

	// rust: both_down_goes_to_juggernaut_label
	@Test
	public void bothDownGoesToJuggernautLabel() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.GOTO_LABEL_ON_JUGGERNAUT, "jug"));
		step.setParameter(StepParameter.from(StepParameterKey.BLOCK_RESULT, BlockResult.BOTH_DOWN));
		assertEquals(StepAction.GOTO_LABEL, GameFixture.startStep(step));
	}

	// rust: pow_defender_falls_goto_pushback
	@Test
	public void powDefenderFallsGotoPushback() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.GOTO_LABEL_ON_PUSHBACK, "push"));
		step.setParameter(StepParameter.from(StepParameterKey.BLOCK_RESULT, BlockResult.POW));
		assertEquals(StepAction.GOTO_LABEL, GameFixture.startStep(step));
		assertEquals(PlayerState.FALLING, stateBase("away1"));
	}

	// rust: pushback_defender_restores_goto_pushback
	@Test
	public void pushbackDefenderRestoresGotoPushback() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.GOTO_LABEL_ON_PUSHBACK, "push"));
		step.setParameter(StepParameter.from(StepParameterKey.BLOCK_RESULT, BlockResult.PUSHBACK));
		step.setParameter(StepParameter.from(StepParameterKey.OLD_DEFENDER_STATE, new PlayerState(PlayerState.PRONE)));
		assertEquals(StepAction.GOTO_LABEL, GameFixture.startStep(step));
		assertEquals(PlayerState.PRONE, stateBase("away1"));
	}

}
