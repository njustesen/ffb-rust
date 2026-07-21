package com.fumbbl.ffb.util;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/util/string_tool.rs for {@link StringTool}.
 */
public class StringToolTest {

	@Test
	void bindBasic() {
		assertEquals("Hello World, you are great!",
			StringTool.bind("Hello $1, you are $2!", new Object[] { "World", "great" }));
	}

	@Test
	void bindUnmatchedPlaceholderDropped() {
		// Java StringTool.bind drops placeholders with no matching parameter.
		assertEquals("a and ", StringTool.bind("$1 and $3", new Object[] { "a" }));
	}

	@Test
	void formatThousandsBasic() {
		assertEquals("2,130,000", StringTool.formatThousands(2_130_000));
		assertEquals("1,000", StringTool.formatThousands(1000));
		assertEquals("42", StringTool.formatThousands(42));
	}

	@Test
	void enumeration() {
		assertEquals("a", StringTool.buildEnumeration(new String[] { "a" }));
		assertEquals("a and b", StringTool.buildEnumeration(new String[] { "a", "b" }));
		assertEquals("a, b and c", StringTool.buildEnumeration(new String[] { "a", "b", "c" }));
	}

	@Test
	void isNumberChecks() {
		assertTrue(StringTool.isNumber("42"));
		assertFalse(StringTool.isNumber("4x"));
		assertFalse(StringTool.isNumber(""));
	}

	@Test
	void isProvidedChecks() {
		assertTrue(StringTool.isProvided("hi"));
		assertFalse(StringTool.isProvided(""));
		assertFalse(StringTool.isProvided(null));
	}

}
