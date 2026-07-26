package com.fumbbl.ffb.server.step.bb2025.move_;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.injury.injuryType.InjuryTypeDropDodge;
import com.fumbbl.ffb.server.injury.injuryType.InjuryTypeDropGFI;
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
 * Mirrors the Rust tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2025/move_/step_fall_down.rs} (param + next-step
 * subset). INJURY_TYPE (server InjuryTypeServer) and COORDINATE_FROM are stored via setParameter; the
 * armour/injury dice are preset via installScriptedDice. The publishes-injury-result and
 * publishes-end-turn-for-gfi-drop / pass-block tests inspect published parameters / turn mode and are
 * deferred.
 */
public class StepFallDownBb2025FixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		gameState.getGame().setHomePlaying(true);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MOVE);
	}

	private IStep fallStep() {
		IStep step = GameFixture.createStep(gameState, StepId.FALL_DOWN);
		step.setParameter(StepParameter.from(StepParameterKey.INJURY_TYPE, new InjuryTypeDropGFI()));
		GameFixture.installScriptedDice(gameState, 3, 4, 3, 4, 3, 4);
		return step;
	}

	// rust: set_parameter_injury_type_name_accepted
	@Test
	public void setParameterInjuryTypeAccepted() {
		assertTrue(GameFixture.createStep(gameState, StepId.FALL_DOWN)
			.setParameter(StepParameter.from(StepParameterKey.INJURY_TYPE, new InjuryTypeDropDodge())));
	}

	// rust: set_parameter_coordinate_from_accepted
	@Test
	public void setParameterCoordinateFromAccepted() {
		assertTrue(GameFixture.createStep(gameState, StepId.FALL_DOWN)
			.setParameter(StepParameter.from(StepParameterKey.COORDINATE_FROM, new FieldCoordinate(5, 5))));
	}

	// rust: unrecognised_parameter_returns_false
	@Test
	public void unrecognisedParameterReturnsFalse() {
		assertFalse(GameFixture.createStep(gameState, StepId.FALL_DOWN)
			.setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}

	// rust: start_returns_next_step
	@Test
	public void startReturnsNextStep() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(fallStep()));
	}
}
