package com.fumbbl.ffb.net;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/i_connection_listener.rs tests.
 */
public class IConnectionListenerTest {

	private static class MockListener implements IConnectionListener {
		boolean called;
		boolean result;

		public void connectionEstablished(boolean pSuccessful) {
			called = true;
			result = pSuccessful;
		}
	}

	@Test
	public void listenerCalledOnSuccess() {
		MockListener l = new MockListener();
		l.connectionEstablished(true);
		assertTrue(l.called);
		assertTrue(l.result);
	}

	@Test
	public void listenerCalledOnFailure() {
		MockListener l = new MockListener();
		l.result = true;
		l.connectionEstablished(false);
		assertTrue(l.called);
		assertFalse(l.result);
	}
}
