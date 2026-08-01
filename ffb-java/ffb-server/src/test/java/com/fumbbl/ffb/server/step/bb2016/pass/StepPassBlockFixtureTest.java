package com.fumbbl.ffb.server.step.bb2016.pass;

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
 * Mirror of ffb-rust crates/ffb-engine/src/step/bb2016/pass/step_pass_block.rs (guard + param subset).
 * With no thrower, start() waits (CONTINUE). END_TURN / END_PLAYER_ACTION accepted via setParameter;
 * unknown → false. The pass-block-report / dump-off-hand-over-bomb skip tests need a thrower + action
 * and a report assertion — deferred; GOTO_LABEL_ON_END is init-consumed (setParameter twin exempt).
 */
public class StepPassBlockFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep newStep() {
		return GameFixture.createStep(gameState, StepId.PASS_BLOCK);
	}

	// rust: no_thrower_returns_continue_without_report
	@Test
	public void noThrowerReturnsContinue() {
		assertEquals(StepAction.CONTINUE, GameFixture.startStep(newStep()));
	}

	// rust: set_parameter_end_turn
	@Test
	public void setParameterEndTurnAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}

	// rust: set_parameter_end_player_action
	@Test
	public void setParameterEndPlayerActionAccepted() {
		assertTrue(newStep().setParameter(StepParameter.from(StepParameterKey.END_PLAYER_ACTION, true)));
	}

	// unrecognised parameter returns false
	@Test
	public void unknownParameterReturnsFalse() {
		assertFalse(newStep().setParameter(StepParameter.from(StepParameterKey.ADMIN_MODE, true)));
	}
}
