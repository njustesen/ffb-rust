package com.fumbbl.ffb.server.step.bb2016.move_;

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
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2016/move_/step_tentacles.rs} (param subset).
 * GOTO_LABEL_ON_SUCCESS is init-consumed (setParameter returns false) so that set_parameter test is
 * exempt. The using-tentacles decision / select-player / decline-reroll / shadowing-roll-report tests
 * run through Java's executeStepHooks (tentacles skill behaviour) with command-driven state and are
 * deferred (hook + use-skill-command driven, not expressible through the headless step fixture).
 */
public class StepTentaclesFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.TENTACLES);
	}

	// rust: coordinate_from_parameter_accepted
	@Test
	public void coordinateFromParameterAccepted() {
		assertTrue(newStep().setParameter(
			StepParameter.from(StepParameterKey.COORDINATE_FROM, new FieldCoordinate(5, 5))));
	}

	// rust: unrecognised_parameter_returns_false
	@Test
	public void unrecognisedParameterReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}
}
