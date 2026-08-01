package com.fumbbl.ffb.server.step.bb2020.multiblock;

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

/**
 * Mirror of ffb-rust crates/ffb-engine/src/step/.../step_report_stab_injury.rs (guard subset). With no
 * injury result, start() → NEXT_STEP (with an injury result it also reports it, then NEXT_STEP). Any
 * setParameter key returns false — PLAYER_ID / INJURY_RESULT are init-consumed (their Rust setParameter
 * twins are exempt); the with-injury report + handle-command tests are report/command-driven and deferred.
 */
public class StepReportStabInjuryFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.REPORT_STAB_INJURY);
	}

	// rust: start_returns_next_step_when_no_injury
	@Test
	public void startReturnsNextStepWhenNoInjury() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// rust: set_parameter_unknown_returns_false
	@Test
	public void setParameterUnknownReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}
}
