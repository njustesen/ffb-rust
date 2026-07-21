package com.fumbbl.ffb.inducement.bb2025;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/inducement/bb2025/inducement_collection.rs for {@link InducementCollection}.
 */
public class InducementCollectionTest {

	@Test
	public void bb2025HasTeamMascot() {
		InducementCollection c = new InducementCollection();
		assertTrue(c.getTypes().stream().anyMatch(t -> "teamMascot".equals(t.getName())));
	}

	@Test
	public void bb2025HasThrowARock() {
		InducementCollection c = new InducementCollection();
		assertTrue(c.getTypes().stream().anyMatch(t -> "throwARock".equals(t.getName())));
	}

	@Test
	public void totalCountIsAtLeast16() {
		InducementCollection c = new InducementCollection();
		assertTrue(c.getTypes().size() >= 16);
	}
}
