package com.fumbbl.ffb.server.step.action.block;

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
 * {@code ffb-rust/crates/ffb-engine/src/step/action/block/step_dauntless.rs} (param subset + the
 * no-Dauntless fall-through). The strength-comparison skip-roll, dice-driven roll, indomitable and
 * report tests need attacker+defender block-target setup and are dice/command-driven — deferred, as
 * with the other hook-delegating block steps (StepBlockChainsaw). USING_STAB/VOMIT/CHAINSAW/
 * BREATHE_FIRE are accepted via setParameter; unrecognised keys return false.
 */
public class StepDauntlessFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.placePlayer(gameState, "away1", 5, 6);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.BLOCK);
		// StepDauntless reads the block defender's strength, so a defender must be selected.
		gameState.getGame().setDefenderId("away1");
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.DAUNTLESS);
	}

	// rust: no_dauntless_skill_returns_next
	@Test
	public void noDauntlessSkillReturnsNext() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// rust: set_parameter_stores_flags (USING_STAB/VOMIT/CHAINSAW/BREATHE_FIRE accepted)
	@Test
	public void setParameterAcceptsUsingFlags() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.USING_STAB, true)));
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.USING_VOMIT, true)));
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.USING_CHAINSAW, true)));
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.USING_BREATHE_FIRE, true)));
	}

	// rust: unknown_parameter_returns_false
	@Test
	public void unknownParameterReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}
}
