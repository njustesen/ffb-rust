package com.fumbbl.ffb.client;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class TextStyleTest {

	@Test
	void getNameNoneIsEmpty() {
		assertEquals("", TextStyle.NONE.getName());
	}

	@Test
	void getNameBold() {
		assertEquals("bold", TextStyle.BOLD.getName());
	}

	@Test
	void getNameHomeBold() {
		assertEquals("homeBold", TextStyle.HOME_BOLD.getName());
	}

	@Test
	void getNameMention() {
		assertEquals("mention", TextStyle.MENTION.getName());
	}
}
