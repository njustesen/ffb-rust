package com.fumbbl.ffb.client.util;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class UtilClientChatTest {

	@Test
	void replaceRangeMiddle() {
		assertEquals("hello there", UtilClientChat.replaceRange("hello world", 6, 11, "there"));
	}

	@Test
	void replaceRangeInsertAtEnd() {
		assertEquals("hello world", UtilClientChat.replaceRange("hello", 5, 5, " world"));
	}

	@Test
	void replaceRangeInsertAtStart() {
		assertEquals("hello world", UtilClientChat.replaceRange("world", 0, 0, "hello "));
	}

	@Test
	void replaceRangeEmptyInsertionDeletes() {
		assertEquals("hello", UtilClientChat.replaceRange("hello world", 5, 11, ""));
	}
}
