package com.fumbbl.ffb.server.step.action.block;

import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.ActingPlayer;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.PlayerResult;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepAction;
import com.fumbbl.ffb.server.step.StepId;
import com.fumbbl.ffb.server.step.StepParameter;
import com.fumbbl.ffb.server.step.StepParameterKey;
import com.fumbbl.ffb.server.step.StepParameterSet;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/step/action/block/step_block_statistics.rs. On the first
 * block of a player's action it marks the acting player hasBlocked, starts the turn, clears
 * concession-possible, and adds `increment` (default 1) to the player's block count; subsequent calls
 * are a no-op. INCREMENT is an init param (Rust threads it via setParameter — a structural detail);
 * PLAYER_ID_TO_REMOVE decrements the increment. (The Rust no_acting_player_returns_next guard is
 * Rust-defensive — Java always has an acting player during a block — exempt.)
 */
public class StepBlockStatisticsFixtureTest {

	private GameState gameState;
	private Game game;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		game = gameState.getGame();
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.BLOCK);
		game.setConcessionPossible(true);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.BLOCK_STATISTICS);
	}

	private IStep stepWithIncrement(int increment) {
		IStep step = newStep();
		StepParameterSet set = new StepParameterSet();
		set.add(StepParameter.from(StepParameterKey.INCREMENT, increment));
		step.init(set);
		return step;
	}

	private ActingPlayer actingPlayer() {
		return game.getActingPlayer();
	}

	private PlayerResult homePlayerResult() {
		return game.getGameResult().getPlayerResult(game.getPlayerById("home1"));
	}

	// rust: first_block_sets_has_blocked_and_turn_started
	@Test
	public void firstBlockSetsHasBlockedAndTurnStarted() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
		assertTrue(actingPlayer().hasBlocked());
		assertTrue(game.getTurnData().isTurnStarted());
	}

	// rust: first_block_clears_concession_possible
	@Test
	public void firstBlockClearsConcessionPossible() {
		GameFixture.startStep(newStep());
		assertFalse(game.isConcessionPossible());
	}

	// rust: first_block_increments_player_block_count
	@Test
	public void firstBlockIncrementsPlayerBlockCount() {
		GameFixture.startStep(newStep());
		assertEquals(1, homePlayerResult().getBlocks());
	}

	// rust: second_call_skips_stats
	@Test
	public void secondCallSkipsStats() {
		GameFixture.startStep(newStep());
		GameFixture.startStep(newStep());
		assertEquals(1, homePlayerResult().getBlocks());
	}

	// rust: increment_can_be_set_via_parameter
	@Test
	public void incrementCanBeSetViaParameter() {
		GameFixture.startStep(stepWithIncrement(3));
		assertEquals(3, homePlayerResult().getBlocks());
	}

	// rust: player_id_to_remove_decrements_increment
	@Test
	public void playerIdToRemoveDecrementsIncrement() {
		IStep step = newStep();
		assertTrue(step.setParameter(StepParameter.from(StepParameterKey.PLAYER_ID_TO_REMOVE, "x")));
		GameFixture.startStep(step);
		assertEquals(0, homePlayerResult().getBlocks());
	}
}
