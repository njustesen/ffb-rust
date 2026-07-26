package com.fumbbl.ffb.server.step.bb2016;

import com.fumbbl.ffb.ApothecaryMode;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepAction;
import com.fumbbl.ffb.server.step.StepId;
import com.fumbbl.ffb.server.step.StepParameter;
import com.fumbbl.ffb.server.step.StepParameterKey;
import com.fumbbl.ffb.server.step.StepParameterSet;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust tests in
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2016/step_apothecary.rs} (mode-conditional param +
 * no-injury subset). The apothecary mode is mandatory and consumed at init() (not via setParameter),
 * so apothecary_mode_parameter_accepted is exempt (init-consumed) and the mode is supplied here by
 * calling init() with an APOTHECARY_MODE parameter set. default_show_report reads a private field and
 * the apothecary-choice / regeneration-save / apothecary-roll-report tests are command/report driven
 * and deferred.
 */
public class StepApothecaryFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep stepWithMode(ApothecaryMode mode) {
		IStep step = GameFixture.createStep(gameState, StepId.APOTHECARY);
		StepParameterSet set = new StepParameterSet();
		set.add(StepParameter.from(StepParameterKey.APOTHECARY_MODE, mode));
		step.init(set);
		return step;
	}

	// rust: defender_poisoned_only_accepted_in_defender_mode
	@Test
	public void defenderPoisonedOnlyAcceptedInDefenderMode() {
		assertTrue(stepWithMode(ApothecaryMode.DEFENDER)
			.setParameter(StepParameter.from(StepParameterKey.DEFENDER_POISONED, true)));
		assertFalse(stepWithMode(ApothecaryMode.ATTACKER)
			.setParameter(StepParameter.from(StepParameterKey.DEFENDER_POISONED, true)));
	}

	// rust: attacker_poisoned_only_accepted_in_attacker_mode
	@Test
	public void attackerPoisonedOnlyAcceptedInAttackerMode() {
		assertTrue(stepWithMode(ApothecaryMode.ATTACKER)
			.setParameter(StepParameter.from(StepParameterKey.ATTACKER_POISONED, true)));
		assertFalse(stepWithMode(ApothecaryMode.DEFENDER)
			.setParameter(StepParameter.from(StepParameterKey.ATTACKER_POISONED, true)));
	}

	// rust: using_piling_on_false_suppresses_report_in_defender_mode
	@Test
	public void usingPilingOnFalseAcceptedInDefenderMode() {
		assertTrue(stepWithMode(ApothecaryMode.DEFENDER)
			.setParameter(StepParameter.from(StepParameterKey.USING_PILING_ON, false)));
	}

	// rust: using_piling_on_not_accepted_in_attacker_mode
	@Test
	public void usingPilingOnNotAcceptedInAttackerMode() {
		assertFalse(stepWithMode(ApothecaryMode.ATTACKER)
			.setParameter(StepParameter.from(StepParameterKey.USING_PILING_ON, false)));
	}

	// rust: no_injury_result_returns_next
	@Test
	public void noInjuryResultReturnsNext() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(stepWithMode(ApothecaryMode.DEFENDER)));
	}
}
