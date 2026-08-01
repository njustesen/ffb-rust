package com.fumbbl.ffb.server.step.mixed;

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

/**
 * Mirror of ffb-rust crates/ffb-engine/src/step/mixed/step_pro.rs (param subset). StepPro has no
 * setParameter keys (PLAYER_ID is init-consumed, not setParameter), so any key returns false — the
 * Rust set_parameter_player_id twin is EXEMPT (init-consumed). The Pro reroll roll (4+/3-min) /
 * SUCCESSFUL_PRO publish / used-pro-marking / decline-reroll tests are dice/reroll-source-driven and
 * deferred; no_player_id dereferences a null player in Java — Rust-defensive, deferred.
 */
public class StepProFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
	}

	// rust: set_parameter_rejects_unknown
	@Test
	public void setParameterRejectsUnknown() {
		IStep step = GameFixture.createStep(gameState, StepId.PRO);
		assertFalse(step.setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}
}
