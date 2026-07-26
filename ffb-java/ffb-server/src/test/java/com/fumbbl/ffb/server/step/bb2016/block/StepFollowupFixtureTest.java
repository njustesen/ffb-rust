package com.fumbbl.ffb.server.step.bb2016.block;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.PlayerState;
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
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2016/block/step_followup.rs} (param/branch subset).
 * The publishes-CoordinateFrom test (no published-param accessor) and the Fend / Juggernaut
 * skill-report tests (need placed defender/attacker with skills + report inspection) are deferred.
 * The Rust no_choice_no_defender_stays_cont test is EXEMPT — Java's fend-dialog path dereferences
 * game.getDefender() with no null-guard (a defender is always present here in production).
 */
public class StepFollowupFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		// StepFollowup reads the acting player's state at start — place one (present in production).
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.BLOCK);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.FOLLOWUP);
	}

	// rust: set_parameter_accepted
	@Test
	public void setParameterAccepted() {
		IStep step = newStep();
		FieldCoordinate coord = new FieldCoordinate(3, 7);
		assertTrue(step.setParameter(StepParameter.from(StepParameterKey.DEFENDER_POSITION, coord)));
		assertTrue(step.setParameter(StepParameter.from(StepParameterKey.FOLLOWUP_CHOICE, true)));
		assertTrue(step.setParameter(
			StepParameter.from(StepParameterKey.OLD_DEFENDER_STATE, new PlayerState(PlayerState.STANDING))));
		assertTrue(step.setParameter(StepParameter.from(StepParameterKey.COORDINATE_FROM, coord)));
	}

	// rust: multiple_block_forces_no_followup
	@Test
	public void multipleBlockForcesNoFollowup() {
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.MULTIPLE_BLOCK);
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// rust: followup_true_returns_next_step
	@Test
	public void followupTrueReturnsNextStep() {
		IStep step = newStep();
		step.setParameter(StepParameter.from(StepParameterKey.FOLLOWUP_CHOICE, true));
		step.setParameter(StepParameter.from(StepParameterKey.DEFENDER_POSITION, new FieldCoordinate(6, 5)));
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(step));
	}
}
