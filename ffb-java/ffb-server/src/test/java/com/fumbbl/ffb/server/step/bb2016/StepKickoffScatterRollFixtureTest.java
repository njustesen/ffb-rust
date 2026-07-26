package com.fumbbl.ffb.server.step.bb2016;

import com.fumbbl.ffb.FieldCoordinate;
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
 * Mirrors the Rust tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2016/step_kickoff_scatter_roll.rs} (param + ball-moving
 * subset). KICKOFF_START_COORDINATE is stored via setParameter. With a start coordinate and no
 * kicking player (so no kick-skill dialog), the scatter phase (scatter dice via installScriptedDice)
 * sets the ball moving. no_start_coord_returns_next_step is exempt (Rust guards a missing start
 * coordinate; Java's findScatterCoordinate dereferences the null start -> NPE). The publishes-touchback
 * / kicking-player-coordinate / kickoff-bounds / use-kick-choice-halves / report / d6-vs-d8 /
 * phase-1-direction tests inspect published params, reports, or private roll state and are deferred.
 */
public class StepKickoffScatterRollFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		gameState.getGame().setHomePlaying(true);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.KICKOFF_SCATTER_ROLL);
	}

	// rust: set_parameter_kickoff_start_coord_accepted
	@Test
	public void setParameterKickoffStartCoordinateAccepted() {
		assertTrue(newStep().setParameter(
			StepParameter.from(StepParameterKey.KICKOFF_START_COORDINATE, new FieldCoordinate(13, 7))));
	}

	// rust: set_parameter_unknown_returns_false
	@Test
	public void setParameterUnknownReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}

	// rust: sets_ball_moving
	@Test
	public void setsBallMoving() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.KICKOFF_START_COORDINATE, new FieldCoordinate(13, 7)));
		GameFixture.installScriptedDice(gameState, 1, 3, 1, 3, 1, 3);
		GameFixture.startStep(step);
		assertTrue(gameState.getGame().getFieldModel().isBallMoving());
	}
}
