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
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2020/block/step_block_chainsaw.rs} (param + next-step
 * subset). USING_CHAINSAW is stored via setParameter. An acting player that is not using a chainsaw,
 * or is using one but lacks the blocksLikeChainsaw skill, falls straight through to NEXT_STEP (the
 * placed acting player has no chainsaw skill). The successful-hit / failed-hit tests publish the
 * drop-player / steady-footing contexts and are deferred.
 */
public class StepBlockChainsawBb2020FixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		gameState.getGame().setHomePlaying(true);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.BLOCK);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.BLOCK_CHAINSAW);
	}

	// rust: set_parameter_using_chainsaw_accepted
	@Test
	public void setParameterUsingChainsawAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.USING_CHAINSAW, true)));
	}

	// rust: not_using_chainsaw_returns_next_step
	@Test
	public void notUsingChainsawReturnsNextStep() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// rust: using_chainsaw_but_player_has_no_skill_returns_next_step
	@Test
	public void usingChainsawButPlayerHasNoSkillReturnsNextStep() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.USING_CHAINSAW, true));
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(step));
	}
}
