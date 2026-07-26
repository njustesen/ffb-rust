package com.fumbbl.ffb.server.step.bb2016.ttm;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.fixture.GeneratorTestSupport;
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
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2016/ttm/step_end_scatter_player.rs}.
 * A THROWN_PLAYER_ID that does not resolve to a real player suppresses the ScatterPlayer push.
 * The two IS_KICKED_PLAYER "publishes param" tests are deferred (no published-param accessor).
 */
public class StepEndScatterPlayerFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.END_SCATTER_PLAYER);
	}

	// rust: nonexistent_thrown_player_returns_next_without_push
	@Test
	public void nonexistentThrownPlayerReturnsNextWithoutPush() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.THROWN_PLAYER_ID, "ghost"));
		step.setParameter(StepParameter.from(StepParameterKey.THROWN_PLAYER_STATE, new PlayerState(0)));
		step.setParameter(StepParameter.from(StepParameterKey.THROWN_PLAYER_COORDINATE, new FieldCoordinate(5, 5)));
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(step));
		assertEquals(0, GeneratorTestSupport.sequence(gameState).length);
	}

	// rust: missing_state_returns_next_without_push
	@Test
	public void missingStateReturnsNextWithoutPush() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.THROWN_PLAYER_ID, "p1"));
		step.setParameter(StepParameter.from(StepParameterKey.THROWN_PLAYER_COORDINATE, new FieldCoordinate(5, 5)));
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(step));
	}

	// rust: set_parameter_thrown_player_id
	@Test
	public void setParameterThrownPlayerId() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.THROWN_PLAYER_ID, "p1")));
	}
}
