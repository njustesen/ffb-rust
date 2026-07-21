package com.fumbbl.ffb.util;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

import org.junit.jupiter.api.Test;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/util/list_tool.rs for {@link ListTool}.
 */
public class ListToolTest {

	@Test
	void firstElementEmptyReturnsNone() {
		assertNull(ListTool.firstElement(new ArrayList<>()));
	}

	@Test
	void addAllAppends() {
		List<String> v = new ArrayList<>(Arrays.asList("a"));
		boolean added = ListTool.addAll(v, new String[] { "b", "c" });
		assertTrue(added);
		assertEquals(Arrays.asList("a", "b", "c"), v);
	}

	@Test
	void replaceAllClearsFirst() {
		List<String> v = new ArrayList<>(Arrays.asList("old"));
		ListTool.replaceAll(v, new String[] { "new" });
		assertEquals(Arrays.asList("new"), v);
	}

}
