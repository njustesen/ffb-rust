package com.fumbbl.ffb.server.step.bb2025.kickoff;

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
 * Mirrors the Rust tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2025/kickoff/step_kickoff_scatter_roll.rs} (param +
 * action subset). KICKOFF_START_COORDINATE is stored via setParameter. With no start coordinate the
 * step waits for input (CONTINUE); with a start coordinate the scatter resolves and returns NEXT_STEP.
 * The publishes-touchback / kicking-coordinate / ball-placed / touchback-wrong-half / report tests
 * inspect published params, field placement, or reports and are deferred.
 */
public class StepKickoffScatterRollBb2025FixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		gameState.getGame().setHomePlaying(true);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.KICKOFF_SCATTER_ROLL);
	}

	// rust: set_parameter_accepts_kickoff_start_coordinate
	@Test
	public void setParameterAcceptsKickoffStartCoordinate() {
		assertTrue(newStep().setParameter(
			StepParameter.from(StepParameterKey.KICKOFF_START_COORDINATE, new FieldCoordinate(13, 7))));
	}

	// rust: start_without_coordinate_returns_cont — EXEMPT: Rust returns CONTINUE with no start
	// coordinate, but Java's executeStep asserts a non-null startCoordinate (IllegalArgumentException).

	// rust: start_with_coordinate_returns_next_step
	@Test
	public void startWithCoordinateReturnsNextStep() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.KICKOFF_START_COORDINATE, new FieldCoordinate(13, 7)));
		GameFixture.installScriptedDice(gameState, 1, 3, 1, 3, 1, 3, 1, 3);
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(step));
	}
}
