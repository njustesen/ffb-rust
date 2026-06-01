package com.fumbbl.ffb.server.util;

import com.fumbbl.ffb.util.StringTool;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class StringToolTest {

    @Test
    void bind_replaces_dollar_one_placeholder() {
        assertEquals("Hello World", StringTool.bind("Hello $1", "World"));
    }

    @Test
    void bind_two_placeholders() {
        assertEquals("a and b", StringTool.bind("$1 and $2", "a", "b"));
    }

    @Test
    void format_thousands_large_number() {
        assertEquals("2,130,000", StringTool.formatThousands(2130000L));
    }

    @Test
    void build_enumeration_with_two_items() {
        assertEquals("a and b", StringTool.buildEnumeration(new String[]{"a", "b"}));
    }

    @Test
    void is_number_returns_false_for_text() {
        assertFalse(StringTool.isNumber("abc"));
    }
}
