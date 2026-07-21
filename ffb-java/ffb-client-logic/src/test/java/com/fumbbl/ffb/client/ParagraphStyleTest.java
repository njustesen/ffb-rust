package com.fumbbl.ffb.client;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class ParagraphStyleTest {

	@Test
	void getNameIndent0() {
		assertEquals("indent0", ParagraphStyle.INDENT_0.getName());
	}

	@Test
	void getNameIndent6() {
		assertEquals("indent6", ParagraphStyle.INDENT_6.getName());
	}

	@Test
	void getNameSpaceAboveBelow() {
		assertEquals("spaceAboveBelow", ParagraphStyle.SPACE_ABOVE_BELOW.getName());
	}

	@Test
	void getNameChatBody() {
		assertEquals("chatBody", ParagraphStyle.CHAT_BODY.getName());
	}
}
