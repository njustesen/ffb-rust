package com.fumbbl.ffb.client.state.logic;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertSame;

class ClientActionTest {

	@Test
	public void testMoveVariantEqualsItself() {
		assertEquals(ClientAction.MOVE, ClientAction.MOVE);
	}

	@Test
	public void testDistinctVariantsAreNotEqual() {
		assertNotEquals(ClientAction.MOVE, ClientAction.BLOCK);
	}

	@Test
	public void testVariantIsStableAcrossReferences() {
		ClientAction a = ClientAction.PASS;
		ClientAction b = a;
		assertSame(a, b);
		assertEquals(a, b);
	}
}
