package com.fumbbl.ffb.mechanics;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;

/**
 * Mirror of ffb-rust crates/ffb-mechanics/src/pass_result.rs tests.
 */
public class PassResultTest {

	// rust: accurate_name
	@Test
	public void accurateName() {
		assertEquals("ACCURATE", PassResult.ACCURATE.getName());
	}

	// rust: fumble_name
	@Test
	public void fumbleName() {
		assertEquals("FUMBLE", PassResult.FUMBLE.getName());
	}

	// rust: variants_are_distinct
	@Test
	public void variantsAreDistinct() {
		assertNotEquals(PassResult.ACCURATE, PassResult.INACCURATE);
	}

	// rust: all_variants_have_names
	@Test
	public void allVariantsHaveNames() {
		assertEquals("SAVED_FUMBLE", PassResult.SAVED_FUMBLE.getName());
		assertEquals("WILDLY_INACCURATE", PassResult.WILDLY_INACCURATE.getName());
		assertEquals("INACCURATE", PassResult.INACCURATE.getName());
	}

	// rust: clone_and_copy_preserve_equality (Java enums are singletons; Rust Copy/Clone -> identity)
	@Test
	public void cloneAndCopyPreserveEquality() {
		PassResult r = PassResult.ACCURATE;
		PassResult c = r;
		assertEquals(r, c);
	}
}
