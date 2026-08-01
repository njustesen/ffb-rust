package com.fumbbl.ffb.server.step.bb2025.kickoff;

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
 * Mirror of ffb-rust crates/ffb-engine/src/step/bb2025/kickoff/step_kickoff_scatter_roll_ask_after.rs
 * (param subset). KICKOFF_START_COORDINATE is consumed via setParameter (return true); unrecognised keys
 * return false. The scatter roll / ask-after-scatter dialog is dice/command driven and deferred.
 */
public class StepKickoffScatterRollAskAfterFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.KICKOFF_SCATTER_ROLL_ASK_AFTER);
	}

	// rust: KickoffStartCoordinate accepted
	@Test
	public void setParameterKickoffStartCoordinateAccepted() {
		assertTrue(newStep().setParameter(
			StepParameter.from(StepParameterKey.KICKOFF_START_COORDINATE, new FieldCoordinate(13, 7))));
	}

	// unrecognised parameter returns false
	@Test
	public void unknownParameterReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}
}
