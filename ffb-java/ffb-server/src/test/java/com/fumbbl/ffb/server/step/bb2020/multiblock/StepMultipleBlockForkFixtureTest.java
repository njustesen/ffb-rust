package com.fumbbl.ffb.server.step.bb2020.multiblock;

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
 * Mirror of ffb-rust crates/ffb-engine/src/step/bb2020/multiblock/step_multiple_block_fork.rs (param
 * subset). PLAYER_ID_TO_REMOVE is accepted via setParameter (it drops the named target from the fork's
 * target list — a no-op removal when the list is empty, still returns true); unrecognised keys fall
 * through to super and return false. The fork's target-selection / block-sequence wiring is hook/command
 * driven and deferred.
 */
public class StepMultipleBlockForkFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.MULTI_BLOCK_FORK);
	}

	// rust: player_id_to_remove_removes_target (accepted even with no targets configured)
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
