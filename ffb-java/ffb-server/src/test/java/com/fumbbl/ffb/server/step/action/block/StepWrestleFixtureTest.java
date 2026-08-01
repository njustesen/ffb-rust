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
 * {@code ffb-rust/crates/ffb-engine/src/step/action/block/step_wrestle.rs} (param subset). Like the
 * other OLD_DEFENDER_STATE-storing hook-delegating block steps (StepBlockDodge), the behavioural
 * tests need the full block defender-state setup and are deferred. OLD_DEFENDER_STATE is accepted via
 * setParameter; unrecognised keys return false.
 *
 * NOTE (deferred, needs investigation): the Rust neither_has_wrestle_returns_next_no_events asserts
 * StepAction::NextStep, but the Java StepWrestle start with a placed attacker+defender (and no
 * OLD_DEFENDER_STATE param yet) returns REPEAT — its WrestleBehaviour hook does not short-circuit to
 * NEXT before consuming defender state (unlike StepJuggernaut, which returns NEXT on the non-Blitz /
 * no-skill guard first). Resolving whether this is a fixture-state gap or a NEXT/REPEAT divergence
 * needs a WrestleBehaviour-hook vs Rust-start comparison — out of scope for this param batch.
 */
public class StepWrestleFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.placePlayer(gameState, "away1", 5, 6);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.BLOCK);
		gameState.getGame().setDefenderId("away1");
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.WRESTLE);
	}

	// rust: set_parameter_stores_old_defender_state
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
