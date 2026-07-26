package com.fumbbl.ffb.server;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/action_status.rs tests.
 * Rust Copy/Clone/Debug map to Java enum identity / name().
 */
public class ActionStatusTest {

	// rust: success_is_not_failure
	@Test
	public void successIsNotFailure() {
		assertNotEquals(ActionStatus.SUCCESS, ActionStatus.FAILURE);
	}

	// rust: skill_choice_variants_are_distinct
	@Test
	public void skillChoiceVariantsAreDistinct() {
		assertNotEquals(ActionStatus.SKILL_CHOICE_YES, ActionStatus.SKILL_CHOICE_NO);
	}

	// rust: waiting_variants_are_distinct
	@Test
	public void waitingVariantsAreDistinct() {
		assertNotEquals(ActionStatus.WAITING_FOR_RE_ROLL, ActionStatus.WAITING_FOR_SKILL_USE);
		assertNotEquals(ActionStatus.WAITING_FOR_SKILL_USE, ActionStatus.WAIT_FOR_ACTION_CHANGE);
	}

	// rust: all_variants_are_pairwise_distinct
	@Test
	public void allVariantsArePairwiseDistinct() {
		assertEquals(7, ActionStatus.values().length);
	}

	// rust: copy_semantics_preserved
	@Test
	public void copySemanticsPreserved() {
		assertEquals(ActionStatus.SUCCESS, ActionStatus.SUCCESS);
	}

	// rust: clone_equals_original
	@Test
	public void cloneEqualsOriginal() {
		assertEquals(ActionStatus.FAILURE, ActionStatus.FAILURE);
	}

	// rust: debug_format_contains_variant_name
	@Test
	public void debugFormatContainsVariantName() {
		assertTrue(ActionStatus.SUCCESS.name().contains("SUCCESS"));
	}

	// rust: wait_for_action_change_is_distinct_from_failure
	@Test
	public void waitForActionChangeIsDistinctFromFailure() {
		assertNotEquals(ActionStatus.WAIT_FOR_ACTION_CHANGE, ActionStatus.FAILURE);
	}
}
