package com.fumbbl.ffb.server.step.mixed.multiblock;

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
 * Mirror of ffb-rust crates/ffb-engine/src/step/mixed/multiblock/step_dispatch_dump_off.rs (guard).
 * With no ball-carrier among the (init-supplied) targets, the step is a NEXT_STEP no-op. The
 * ball-carrier-in-targets dump-off sequence push (+ setDefenderId + DEFENDER_POSITION publish) and the
 * ball-in-play / ball-moving / remove-target tests need placed ball-carriers and are deferred.
 */
public class StepDispatchDumpOffFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.DISPATCH_DUMP_OFF);
	}

	// rust: no_ball_carrier_in_targets_is_noop (no targets → NEXT_STEP)
	@Test
	public void noTargetsIsNoopReturnsNext() {
		assertEquals(StepAction.NEXT_STEP, GameFixture.startStep(newStep()));
	}

	// unrecognised parameter returns false
	@Test
	public void unknownParameterReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}
}
