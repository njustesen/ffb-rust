package com.fumbbl.ffb.util;

import static org.junit.jupiter.api.Assertions.assertEquals;

import org.junit.jupiter.api.Test;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/util/util_url.rs for {@link UtilUrl}.
 */
public class UtilUrlTest {

	@Test
	void relativeAppendedToBaseDir() {
		String result = UtilUrl.createUrl("http://fumbbl.com/FUMBBL/Images/", "PlayerIcons/amlineman1.gif");
		assertEquals("http://fumbbl.com/FUMBBL/Images/PlayerIcons/amlineman1.gif", result);
	}

	@Test
	void absoluteRelativeWins() {
		String result = UtilUrl.createUrl("http://fumbbl.com/", "http://google.de/icon.gif");
		assertEquals("http://google.de/icon.gif", result);
	}

	@Test
	void emptyBaseReturnsRelative() {
		String result = UtilUrl.createUrl(null, "icon.gif");
		assertEquals("icon.gif", result);
	}

}
