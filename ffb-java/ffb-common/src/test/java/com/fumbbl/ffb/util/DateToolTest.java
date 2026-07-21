package com.fumbbl.ffb.util;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.Date;

import org.junit.jupiter.api.Test;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/util/date_tool.rs for {@link DateTool}.
 */
public class DateToolTest {

	@Test
	void isEqualSame() {
		assertTrue(DateTool.isEqual(new Date(1000), new Date(1000)));
	}

	@Test
	void isEqualNoneBoth() {
		assertTrue(DateTool.isEqual(null, null));
	}

	@Test
	void formatAndParseRoundtrip() {
		Date ts = new Date(1_700_000_000_000L);
		String s = DateTool.formatTimestamp(ts);
		Date parsed = DateTool.parseTimestamp(s);
		assertEquals(ts, parsed);
	}

}
