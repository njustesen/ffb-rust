package com.fumbbl.ffb.server.step.action.block;

import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.PlayerState;
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
 * {@code ffb-rust/crates/ffb-engine/src/step/action/block/step_juggernaut.rs} (param subset + the
 * no-Juggernaut / non-Blitz fall-throughs). Juggernaut only triggers on a Blitz; the skill-use
 * prompt / goto-label / publishes / restore-defender-state / report tests are command-driven and
 * deferred. OLD_DEFENDER_STATE is accepted via setParameter (GOTO_LABEL_ON_SUCCESS is init-set →
 * exempt); unrecognised keys return false.
 */
public class StepJuggernautFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.placePlayer(gameState, "away1", 5, 6);
		gameState.getGame().setDefenderId("away1");
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.JUGGERNAUT);
	}

	// rust: no_juggernaut_skill_returns_next (Blitz acting player without Juggernaut)
	@Test
	public void noJuggernautSkillReturnsNext() {
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.BLITZ);
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// rust: non_blitz_action_returns_next
	@Test
	public void nonBlitzActionReturnsNext() {
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.BLOCK);
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// rust: set_parameter_stores_goto_and_old_state (OLD_DEFENDER_STATE portion — GOTO is init-set)
	@Test
	public void setParameterStoresOldDefenderState() {
		assertTrue(newStep().setParameter(
			StepParameter.from(StepParameterKey.OLD_DEFENDER_STATE, new PlayerState(PlayerState.STANDING))));
	}

	// rust: unknown parameter returns false
	@Test
	public void unknownParameterReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}
}
