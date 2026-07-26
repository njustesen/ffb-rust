package com.fumbbl.ffb.server.step.bb2016.pass;

import com.fumbbl.ffb.RulesCollection;
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
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2016/pass/step_pass.rs} (param subset).
 * The GOTO_LABEL_ON_END / GOTO_LABEL_ON_MISSED_PASS params are init-consumed (setParameter returns
 * false) and PASS_RESULT is computed internally (not accepted via setParameter) so those
 * set_parameter tests are exempt. start_returns_next_step_when_no_thrower is exempt too: Java's
 * StepPass.start dequeues a command before executeStep, so with no pending command it returns
 * CONTINUE rather than the Rust NEXT_STEP (command-loop structural divergence). The with-thrower /
 * publishes-params / mechanic-result / pass-roll-report / bomb-throw tests need a placed thrower +
 * dice + report inspection and are deferred.
 */
public class StepPassFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.PASS);
	}

	// rust: set_parameter_catcher_id
	@Test
	public void setParameterCatcherId() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.CATCHER_ID, "p2")));
	}
}
