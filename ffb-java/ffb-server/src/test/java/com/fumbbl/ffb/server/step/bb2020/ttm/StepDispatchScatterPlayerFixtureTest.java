package com.fumbbl.ffb.server.step.bb2020.ttm;

import com.fumbbl.ffb.PlayerState;
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
 * Mirror of ffb-rust crates/ffb-engine/src/step/bb2020/ttm/step_dispatch_scatter_player.rs (param
 * subset). THROWN_PLAYER_ID / THROWN_PLAYER_STATE / THROWN_PLAYER_HAS_BALL / PASS_RESULT /
 * IS_KICKED_PLAYER / OLD_DEFENDER_STATE are accepted via setParameter; unknown keys return false. The
 * fumble / wildly-inaccurate scatter-sequence-push and kick-team-mate-fumble report tests are
 * dice/sequence-driven and deferred. (PASS_RESULT param twin deferred — PassResult has a known
 * name-collision, avoided here.)
 */
public class StepDispatchScatterPlayerFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.DISPATCH_SCATTER_PLAYER);
	}

	// rust: set_parameter_thrown_player_id
	@Test
	public void setParameterThrownPlayerId() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.THROWN_PLAYER_ID, "away1")));
	}

	// rust: set_parameter_old_defender_state
	@Test
	public void setParameterOldDefenderState() {
		assertTrue(newStep().setParameter(
			StepParameter.from(StepParameterKey.OLD_DEFENDER_STATE, new PlayerState(PlayerState.STANDING))));
	}

	// rust: unknown_parameter_returns_false
	@Test
	public void unknownParameterReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}
}
