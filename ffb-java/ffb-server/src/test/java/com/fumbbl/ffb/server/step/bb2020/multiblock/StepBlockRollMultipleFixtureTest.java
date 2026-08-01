package com.fumbbl.ffb.server.step.bb2020.multiblock;

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
 * Mirror of ffb-rust crates/ffb-engine/src/step/bb2020/multiblock/step_block_roll_multiple.rs (param
 * subset). PLAYER_ID_TO_REMOVE / PLAYER_ID_DAUNTLESS_SUCCESS accepted via setParameter; unknown → false.
 * DEFERRED: start_with_no_targets derefs a null player state (NPE without targets set up), and
 * DOUBLE_TARGET_STRENGTH_FOR_PLAYER is NOT a pure store — it returns false unless the named player is
 * already in the block-rolls list (conditional accept), so both need the set-block-targets fixture and
 * are command/dice-driven. Block-roll creation / attacker-team-selection tests likewise deferred.
 */
public class StepBlockRollMultipleFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.BLOCK_ROLL_MULTIPLE);
	}

	// rust: player_id_to_remove accepted
	@Test
	public void setParameterPlayerIdToRemoveAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.PLAYER_ID_TO_REMOVE, "away1")));
	}

	// rust: player_id_dauntless_success accepted
	@Test
	public void setParameterPlayerIdDauntlessSuccessAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.PLAYER_ID_DAUNTLESS_SUCCESS, "away1")));
	}

	// unrecognised parameter returns false
	@Test
	public void unknownParameterReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}
}
