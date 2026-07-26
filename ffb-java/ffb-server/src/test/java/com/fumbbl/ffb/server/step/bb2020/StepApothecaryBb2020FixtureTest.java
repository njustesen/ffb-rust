package com.fumbbl.ffb.server.step.bb2020;

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
 * {@code ffb-rust/crates/ffb-engine/src/step/bb2020/step_apothecary.rs} (mode-conditional param +
 * no-injury subset). The apothecary mode is mandatory and consumed at init() (not via setParameter),
 * so it is supplied here by calling init() with an APOTHECARY_MODE parameter set; the
 * apothecary-mode-consumed test is exempt (init-consumed). The do-request / cure-to-reserve /
 * inducement-igor / regeneration / apothecary-roll/choice-report tests are command/dice/report driven
 * and deferred.
 */
public class StepApothecaryBb2020FixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
	}

	private IStep stepWithMode(ApothecaryMode mode) {
		IStep step = GameFixture.createStep(gameState, StepId.APOTHECARY);
		StepParameterSet set = new StepParameterSet();
		set.add(StepParameter.from(StepParameterKey.APOTHECARY_MODE, mode));
		step.init(set);
		return step;
	}

	// rust: defender_poisoned only accepted in defender mode
	@Test
	public void defenderPoisonedOnlyAcceptedInDefenderMode() {
		assertTrue(stepWithMode(ApothecaryMode.DEFENDER)
			.setParameter(StepParameter.from(StepParameterKey.DEFENDER_POISONED, true)));
		assertFalse(stepWithMode(ApothecaryMode.ATTACKER)
			.setParameter(StepParameter.from(StepParameterKey.DEFENDER_POISONED, true)));
	}

	// rust: attacker_poisoned only accepted in attacker mode
	@Test
	public void attackerPoisonedOnlyAcceptedInAttackerMode() {
		assertTrue(stepWithMode(ApothecaryMode.ATTACKER)
			.setParameter(StepParameter.from(StepParameterKey.ATTACKER_POISONED, true)));
		assertFalse(stepWithMode(ApothecaryMode.DEFENDER)
			.setParameter(StepParameter.from(StepParameterKey.ATTACKER_POISONED, true)));
	}

	// rust: using_piling_on_false_in_defender_mode_suppresses_report
	@Test
	public void usingPilingOnFalseAcceptedInDefenderMode() {
		assertTrue(stepWithMode(ApothecaryMode.DEFENDER)
			.setParameter(StepParameter.from(StepParameterKey.USING_PILING_ON, false)));
	}

	// rust: using_piling_on_false_in_attacker_mode_does_not_suppress
	@Test
	public void usingPilingOnNotAcceptedInAttackerMode() {
		assertFalse(stepWithMode(ApothecaryMode.ATTACKER)
			.setParameter(StepParameter.from(StepParameterKey.USING_PILING_ON, false)));
	}

	// rust: no_injury_result_returns_next_step
	@Test
	public void noInjuryResultReturnsNext() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(stepWithMode(ApothecaryMode.DEFENDER)));
	}
}
