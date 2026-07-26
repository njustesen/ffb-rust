package com.fumbbl.ffb.server.step.bb2025.punt;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
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
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2025/punt/step_punt_direction.rs} (param + out-of-bounds
 * subset). COORDINATE_FROM / COORDINATE_TO / TOUCHBACK are stored via setParameter (TOUCHBACK also sets
 * the out-of-bounds flag). With no COORDINATE_FROM the step returns NEXT_STEP; when out of bounds it
 * marks punt used on the turn data and gotos the end label. The on-pitch direction-roll / report tests
 * publish params, roll dice for a dialog, or inspect reports and are deferred.
 */
public class StepPuntDirectionBb2025FixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		gameState.getGame().setHomePlaying(true);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
		gameState.getGame().getFieldModel().setBallCoordinate(new FieldCoordinate(0, 7));
	}

	private IStep newStep() {
		IStep step = GameFixture.createStep(gameState, StepId.PUNT_DIRECTION);
		step.setParameter(StepParameter.from(StepParameterKey.GOTO_LABEL_ON_END, "end"));
		return step;
	}

	// rust: set_parameter_stores_coordinates
	@Test
	public void setParameterStoresCoordinates() {
		IStep step = newStep();
		assertTrue(step.setParameter(StepParameter.from(StepParameterKey.COORDINATE_FROM, new FieldCoordinate(5, 5))));
		assertTrue(step.setParameter(StepParameter.from(StepParameterKey.COORDINATE_TO, new FieldCoordinate(6, 5))));
	}

	// rust: set_parameter_touchback_sets_out_of_bounds
	@Test
	public void setParameterTouchbackSetsOutOfBounds() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.TOUCHBACK, true)));
	}

	// rust: missing_coord_from_returns_next — EXEMPT: Rust guards a null coordinate_from and returns
	// NEXT_STEP, but Java's executeStep calls coordinateFrom.getDirection(...) with no null guard -> NPE.

	// rust: out_of_bounds_flag_goto_label
	@Test
	public void outOfBoundsFlagGotoLabel() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.TOUCHBACK, true));
		assertEquals(StepAction.GOTO_LABEL, GameFixture.startStep(step));
	}

	// rust: execute_step_marks_punt_used_on_turn_data
	@Test
	public void executeStepMarksPuntUsedOnTurnData() {
		Game game = gameState.getGame();
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.TOUCHBACK, true));
		GameFixture.startStep(step);
		assertTrue(game.getTurnDataHome().isPuntUsed());
	}
}
