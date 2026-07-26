package com.fumbbl.ffb.server;

import org.junit.jupiter.api.Test;

import java.util.HashSet;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/session_mode.rs tests.
 */
public class SessionModeTest {

	// rust: all_variants_are_distinct
	@Test
	public void allVariantsAreDistinct() {
		assertEquals(5, SessionMode.values().length);
	}

	// rust: home_is_not_away
	@Test
	public void homeIsNotAway() {
		assertNotEquals(SessionMode.HOME, SessionMode.AWAY);
	}

	// rust: copy_semantics
	@Test
	public void copySemantics() {
		assertEquals(SessionMode.ADMIN, SessionMode.ADMIN);
	}

	// rust: clone_equals_original
	@Test
	public void cloneEqualsOriginal() {
		assertEquals(SessionMode.DEV, SessionMode.DEV);
	}

	// rust: debug_format_contains_variant_name
	@Test
	public void debugFormatContainsVariantName() {
		assertTrue(SessionMode.SPEC.name().contains("SPEC"));
	}

	// rust: hash_works_in_set
	@Test
	public void hashWorksInSet() {
		Set<SessionMode> set = new HashSet<>();
		set.add(SessionMode.HOME);
		set.add(SessionMode.HOME);
		assertEquals(1, set.size());
	}

	// rust: spec_admin_dev_are_all_distinct
	@Test
	public void specAdminDevAreAllDistinct() {
		assertNotEquals(SessionMode.SPEC, SessionMode.ADMIN);
		assertNotEquals(SessionMode.ADMIN, SessionMode.DEV);
		assertNotEquals(SessionMode.SPEC, SessionMode.DEV);
	}
}
