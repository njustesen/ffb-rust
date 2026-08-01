package com.fumbbl.ffb.server.step.action.ttm;

import com.fumbbl.ffb.FieldCoordinate;
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
 * Mirror of ffb-rust crates/ffb-engine/src/step/action/ttm/step_eat_team_mate.rs (guard + param
 * subset). With no thrown player set, the step is a NEXT_STEP no-op. THROWN_PLAYER_ID /
 * THROWN_PLAYER_COORDINATE are accepted via setParameter. The valid-player injury publish and the
 * ball-at-thrown-coord scatter/END_TURN publish tests are published-param / placement-driven and deferred.
 */
public class StepEatTeamMateFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.EAT_TEAM_MATE);
	}

	// rust: no_thrown_player_does_nothing_returns_next
	@Test
	public void noThrownPlayerDoesNothingReturnsNext() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// rust: thrown_player_id_parameter_accepted
	@Test
	public void thrownPlayerIdParameterAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.THROWN_PLAYER_ID, "away1")));
	}

	// rust: thrown_player_coordinate_parameter_accepted
	@Test
	public void thrownPlayerCoordinateParameterAccepted() {
		assertTrue(newStep().setParameter(
			StepParameter.from(StepParameterKey.THROWN_PLAYER_COORDINATE, new FieldCoordinate(5, 5))));
	}
}
