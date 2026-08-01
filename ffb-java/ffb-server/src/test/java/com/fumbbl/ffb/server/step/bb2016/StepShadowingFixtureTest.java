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
 * Mirror of ffb-rust crates/ffb-engine/src/step/bb2016/step_shadowing.rs (param subset). COORDINATE_FROM
 * / DEFENDER_POSITION / USING_DIVING_TACKLE are accepted via setParameter; unknown → false. The
 * no-coordinate guard / shadower prompt / diving-tackle & turn-mode disables / player-choice / decline
 * tests are hook + command driven and deferred; USING_SHADOWING is command-set (its setParameter twin exempt).
 */
public class StepShadowingFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.SHADOWING);
	}

	// rust: coordinate_from_parameter_accepted
	@Test
	public void coordinateFromParameterAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.COORDINATE_FROM, new FieldCoordinate(5, 5))));
	}

	// rust: defender_position_parameter_accepted
	@Test
	public void defenderPositionParameterAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.DEFENDER_POSITION, new FieldCoordinate(6, 5))));
	}

	// rust: using_diving_tackle accepted via setParameter
	@Test
	public void usingDivingTackleParameterAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.USING_DIVING_TACKLE, true)));
	}

	// unrecognised parameter returns false
	@Test
	public void unknownParameterReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}
}
