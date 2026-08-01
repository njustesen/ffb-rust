package com.fumbbl.ffb.server.factory;

import com.fumbbl.ffb.server.step.StepAction;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/step/framework/step_action_factory.rs tests. forName
 * does a case-insensitive lookup by StepAction.getName(); unknown names return null.
 */
public class StepActionFactoryTest {

	private final StepActionFactory factory = new StepActionFactory();

	// rust: for_name_continue
	@Test
	public void forNameContinue() {
		assertEquals(StepAction.CONTINUE, factory.forName("continue"));
	}

	// rust: for_name_next_step
	@Test
	public void forNameNextStep() {
		assertEquals(StepAction.NEXT_STEP, factory.forName("nextStep"));
	}

	// rust: for_name_case_insensitive
	@Test
	public void forNameCaseInsensitive() {
		assertEquals(StepAction.NEXT_STEP, factory.forName("NEXTSTEP"));
	}

	// rust: for_name_goto_label
	@Test
	public void forNameGotoLabel() {
		assertEquals(StepAction.GOTO_LABEL, factory.forName("gotoLabel"));
	}

	// rust: for_name_unknown_returns_none
	@Test
	public void forNameUnknownReturnsNull() {
		assertNull(factory.forName("notAnAction"));
	}

	// rust: name_for_roundtrip_continue
	@Test
	public void nameForRoundtripContinue() {
		assertEquals(StepAction.CONTINUE, factory.forName(StepAction.CONTINUE.getName()));
	}

	// rust: name_for_roundtrip_goto_label
	@Test
	public void nameForRoundtripGotoLabel() {
		assertEquals(StepAction.GOTO_LABEL, factory.forName(StepAction.GOTO_LABEL.getName()));
	}
}
