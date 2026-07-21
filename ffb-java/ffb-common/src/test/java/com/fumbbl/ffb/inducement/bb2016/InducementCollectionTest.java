package com.fumbbl.ffb.inducement.bb2016;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/inducement/bb2016/inducement_collection.rs for {@link InducementCollection}.
 */
public class InducementCollectionTest {

	@Test
	public void bb2016CollectionHasMoreThanBase() {
		InducementCollection c = new InducementCollection();
		assertTrue(c.getTypes().size() > 5);
	}

	@Test
	public void hasBribesType() {
		InducementCollection c = new InducementCollection();
		assertTrue(c.getTypes().stream().anyMatch(t -> "bribes".equals(t.getName())));
	}

	@Test
	public void hasIgorType() {
		InducementCollection c = new InducementCollection();
		assertTrue(c.getTypes().stream().anyMatch(t -> "igor".equals(t.getName())));
	}
}
