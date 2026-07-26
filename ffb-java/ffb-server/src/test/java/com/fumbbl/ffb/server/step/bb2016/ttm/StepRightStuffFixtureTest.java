package com.fumbbl.ffb.server.step.bb2016.ttm;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.KickTeamMateRange;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepId;
import com.fumbbl.ffb.server.step.StepParameter;
import com.fumbbl.ffb.server.step.StepParameterKey;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2016/ttm/step_right_stuff.rs} (param subset).
 * DROP_THROWN_PLAYER and KTM_MODIFIER (a KickTeamMateRange) are stored via setParameter. The
 * no_thrown_player_returns_next test is exempt (Rust guards a missing thrown player and returns early;
 * Java's executeStep builds a RightStuffContext with the null player -> NPE). The
 * publishes-coordinate-null / minimum-roll-modifier (long-kick +2, swoop -1) / right-stuff-report
 * tests inspect published params, private minimum-roll math, or reports and are deferred.
 */
public class StepRightStuffFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.RIGHT_STUFF);
	}

	// rust: set_parameter_drop_thrown_player
	@Test
	public void setParameterDropThrownPlayer() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.DROP_THROWN_PLAYER, true)));
	}

	// rust: set_parameter_ktm_range
	@Test
	public void setParameterKtmRange() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.KTM_MODIFIER, KickTeamMateRange.SHORT)));
	}
}
