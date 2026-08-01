package com.fumbbl.ffb.server.step.bb2020;

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
 * Mirror of ffb-rust crates/ffb-engine/src/step/bb2020/step_prayers.rs (param subset). TV_HOME / TV_AWAY
 * are consumed via setParameter (return true, storing each team's treasury value); unrecognised keys fall
 * through to super and return false. The team-value-difference prayer roll is dice/inducement driven and
 * unit-tested Rust-side; deferred here.
 */
public class StepPrayersFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.PRAYERS);
	}

	// rust: set_parameter_tv_home_accepted
	@Test
	public void setParameterTvHomeAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.TV_HOME, 1_000_000)));
	}

	// rust: set_parameter_tv_away_accepted
	@Test
	public void setParameterTvAwayAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.TV_AWAY, 900_000)));
	}

	// unrecognised parameter returns false
	@Test
	public void unknownParameterReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}
}
