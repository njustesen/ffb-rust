package com.fumbbl.ffb.server.step.bb2016.ttm;

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
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2016/ttm/step_init_scatter_player.rs} (param +
 * no-player subset). Only IS_KICKED_PLAYER is stored via this step's setParameter switch; with no
 * thrown player (or no coordinate) the step exits early to NEXT_STEP. The set_parameter_throw_scatter
 * and set_parameter_kicked_player_aliases tests are exempt (THROW_SCATTER / KICKED_PLAYER_ID /
 * KICKED_PLAYER_COORDINATE are not accepted by this step's setParameter -> returns false, unlike Rust).
 * The in-bounds-lands-player / out-of-bounds-injury / publishes-nothing tests need a thrown player
 * that this fixture cannot inject via setParameter, plus scatter dice / published-param inspection,
 * and are deferred.
 */
public class StepInitScatterPlayerFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		gameState.getGame().setHomePlaying(true);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.INIT_SCATTER_PLAYER);
	}

	// rust: set_parameter_is_kicked
	@Test
	public void setParameterIsKicked() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.IS_KICKED_PLAYER, true)));
	}

	// rust: no_player_returns_next_with_params
	@Test
	public void noPlayerReturnsNext() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}
}
