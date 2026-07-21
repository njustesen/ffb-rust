package com.fumbbl.ffb.util;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/util/natural_order_comparator.rs
 * for {@link NaturalOrderComparator}.
 */
public class NaturalOrderComparatorTest {

	@Test
	void digitsSortedNumerically() {
		NaturalOrderComparator cmp = new NaturalOrderComparator();
		assertTrue(cmp.compare("pic2", "pic10") < 0);
	}

	@Test
	void equalStringsReturnEqual() {
		NaturalOrderComparator cmp = new NaturalOrderComparator();
		assertEquals(0, cmp.compare("abc", "abc"));
	}

	@Test
	void alphaBeforeNumericPrefix() {
		NaturalOrderComparator cmp = new NaturalOrderComparator();
		assertTrue(cmp.compare("a1", "b1") < 0);
	}

}
