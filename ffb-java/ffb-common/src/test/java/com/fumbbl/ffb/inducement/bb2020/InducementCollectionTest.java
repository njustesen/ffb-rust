package com.fumbbl.ffb.inducement.bb2020;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/inducement/bb2020/inducement_collection.rs for {@link InducementCollection}.
 */
public class InducementCollectionTest {

	@Test
	public void bb2020CollectionHasPrayers() {
		InducementCollection c = new InducementCollection();
		assertTrue(c.getTypes().stream().anyMatch(t -> "prayers".equals(t.getName())));
	}

	@Test
	public void hasBiasedRef() {
		InducementCollection c = new InducementCollection();
		assertTrue(c.getTypes().stream().anyMatch(t -> "biasedRef".equals(t.getName())));
	}

	@Test
	public void totalTypesCountIsAtLeast15() {
		InducementCollection c = new InducementCollection();
		assertTrue(c.getTypes().size() >= 15);
	}
}
