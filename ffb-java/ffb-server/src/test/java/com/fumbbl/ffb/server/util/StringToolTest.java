package com.fumbbl.ffb.server.util;

import com.fumbbl.ffb.util.StringTool;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class StringToolTest {

    @Test
    void bind_basic() {
        assertEquals("Hello World, you are great!", StringTool.bind("Hello $1, you are $2!", "World", "great"));
    }

    @Test
    void bind_unmatched_placeholder_dropped() {
        assertEquals("a and ", StringTool.bind("$1 and $3", "a"));
    }

    @Test
    void format_thousands_basic() {
        assertEquals("2,130,000", StringTool.formatThousands(2130000L));
        assertEquals("1,000", StringTool.formatThousands(1000L));
        assertEquals("42", StringTool.formatThousands(42L));
    }

    @Test
    void enumeration() {
        assertEquals("a", StringTool.buildEnumeration(new String[]{"a"}));
        assertEquals("a and b", StringTool.buildEnumeration(new String[]{"a", "b"}));
        assertEquals("a, b and c", StringTool.buildEnumeration(new String[]{"a", "b", "c"}));
    }

    @Test
    void is_number_checks() {
        assertTrue(StringTool.isNumber("42"));
        assertFalse(StringTool.isNumber("4x"));
        assertFalse(StringTool.isNumber(""));
    }

    @Test
    void is_provided_checks() {
        assertTrue(StringTool.isProvided("hi"));
        assertFalse(StringTool.isProvided(""));
        assertFalse(StringTool.isProvided(null));
    }
}
