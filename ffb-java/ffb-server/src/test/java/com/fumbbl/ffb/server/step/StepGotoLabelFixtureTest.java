package com.fumbbl.ffb.server.step;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/step/step_goto_label.rs. start() jumps to
 * GOTO_LABEL, or to ALTERNATE_GOTO_LABEL when USE_ALTERNATE_LABEL is set. GOTO_LABEL /
 * ALTERNATE_GOTO_LABEL are init params (mandatory GOTO_LABEL); USE_ALTERNATE_LABEL is accepted via
 * setParameter. Exempt: goto_label_param_accepted / alternate_goto_label_param_accepted (Rust threads
 * those via setParameter, but Java init-consumes them → setParameter returns false).
 */
public class StepGotoLabelFixtureTest {

	private GameState gameState;

	@BeforeEach
	public void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2016);
	}

	private IStep newStep(String primary, String alternate) {
		IStep step = GameFixture.createStep(gameState, StepId.GOTO_LABEL);
		StepParameterSet set = new StepParameterSet();
		set.add(StepParameter.from(StepParameterKey.GOTO_LABEL, primary));
		if (alternate != null) {
			set.add(StepParameter.from(StepParameterKey.ALTERNATE_GOTO_LABEL, alternate));
		}
		step.init(set);
		return step;
	}

	// rust: use_alternate_label_param_accepted
	@Test
	public void useAlternateLabelParamAccepted() {
		assertTrue(newStep("primary", null).setParameter(StepParameter.from(StepParameterKey.USE_ALTERNATE_LABEL, true)));
	}

	// rust: start_goes_to_goto_label
	@Test
	public void startGoesToGotoLabel() {
		IStep step = newStep("primary", null);
		assertEquals(StepAction.GOTO_LABEL, GameFixture.startStep(step));
		assertEquals("primary", step.getResult().getNextActionParameter());
	}

	// rust: start_uses_alternate_when_flag_set
	@Test
	public void startUsesAlternateWhenFlagSet() {
		IStep step = newStep("primary", "alt");
		step.setParameter(StepParameter.from(StepParameterKey.USE_ALTERNATE_LABEL, true));
		assertEquals(StepAction.GOTO_LABEL, GameFixture.startStep(step));
		assertEquals("alt", step.getResult().getNextActionParameter());
	}

	// rust: start_uses_primary_when_alternate_flag_false
	@Test
	public void startUsesPrimaryWhenAlternateFlagFalse() {
		IStep step = newStep("primary", "alt");
		assertEquals(StepAction.GOTO_LABEL, GameFixture.startStep(step));
		assertEquals("primary", step.getResult().getNextActionParameter());
	}

	// rust: unrecognised_parameter_returns_false
	@Test
	public void unrecognisedParameterReturnsFalse() {
		assertFalse(newStep("primary", null).setParameter(StepParameter.from(StepParameterKey.END_TURN, true)));
	}
}
