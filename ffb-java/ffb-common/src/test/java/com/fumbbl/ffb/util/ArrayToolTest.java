package com.fumbbl.ffb.util;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/util/array_tool.rs for {@link ArrayTool}.
 */
public class ArrayToolTest {

	@Test
	void totalSumsArray() {
		assertEquals(6, ArrayTool.total(new int[] { 1, 2, 3 }));
		assertEquals(0, ArrayTool.total(new int[] {}));
	}

	@Test
	void joinIntWithComma() {
		// Rust ArrayTool::join_int returns Some("1,2,3"); Java ArrayTool.join returns the same string.
		// (The Rust empty-array -> None case is intentionally omitted: Java returns "" for an empty
		// non-null array, so that assertion does not translate.)
		assertEquals("1,2,3", ArrayTool.join(new int[] { 1, 2, 3 }, ","));
	}

	@Test
	void isEqualArrays() {
		assertTrue(ArrayTool.isEqual(new int[] { 1, 2 }, new int[] { 1, 2 }));
		assertFalse(ArrayTool.isEqual(new int[] { 1, 2 }, new int[] { 1, 3 }));
		assertTrue(ArrayTool.isEqual(new int[] {}, new int[] {}));
	}

}
