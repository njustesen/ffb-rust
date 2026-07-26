package com.fumbbl.ffb.server.step.bb2016.foul;

import com.fumbbl.ffb.PlayerAction;
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
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2016/foul/step_foul.rs} (param + next-step subset).
 * USING_CHAINSAW is stored via setParameter. With attacker + defender placed, the foul resolves the
 * defender injury (armour/injury dice via installScriptedDice) and returns NEXT_STEP. The
 * no_defender_returns_next_step test is exempt (Rust guards a missing defender and returns early;
 * Java's executeStep dereferences the acting player and defender with no null guard -> NPE). The
 * publishes_injury_result / foul_adds_report / no_defender_does_not_add_foul_report tests inspect
 * published params / reports and are deferred.
 */
public class StepFoulFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
		gameState.getGame().setHomePlaying(true);
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.FOUL);
		GameFixture.placePlayer(gameState, "away1", 6, 5);
		gameState.getGame().setDefenderId("away1");
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.FOUL);
	}

	// rust: set_parameter_using_chainsaw_accepted
	@Test
	public void setParameterUsingChainsawAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.USING_CHAINSAW, true)));
	}

	// rust: unrecognised_parameter_returns_false
	@Test
	public void unrecognisedParameterReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}

	// rust: returns_next_step
	@Test
	public void returnsNextStep() {
		GameFixture.installScriptedDice(gameState, 3, 4, 3, 4, 3, 4);
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}
}
