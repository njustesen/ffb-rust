package com.fumbbl.ffb.server.step.mixed.multiblock;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepId;
import com.fumbbl.ffb.server.step.StepParameter;
import com.fumbbl.ffb.server.step.StepParameterKey;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/step/mixed/multiblock/step_dauntless_multiple.rs (param
 * subset). PLAYER_ID_TO_REMOVE is accepted via setParameter (it drops the named id from the shared
 * block-targets set — a no-op removal when the set is empty, still returns true); unrecognised keys fall
 * through to super and return false. The per-target dauntless roll sequence is hook/dice driven and
 * deferred.
 */
public class StepDauntlessMultipleFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.DAUNTLESS_MULTIPLE);
	}

	// rust: player_id_to_remove_shrinks_targets (accepted even with no targets configured)
	@Test
	public void setParameterPlayerIdToRemoveAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.PLAYER_ID_TO_REMOVE, "away1")));
	}

	// unrecognised parameter returns false
	@Test
	public void unknownParameterReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}
}
