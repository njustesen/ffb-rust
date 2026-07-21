package com.fumbbl.ffb.server.net;

import org.eclipse.jetty.websocket.api.Session;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.mock;

/**
 * Mirrors the Rust {@code replay_session_manager} tests. Rust keys on an i64
 * session handle; Java keys on the Jetty {@link Session}.
 *
 * <p>The Rust sender-registry tests ({@code register_sender_and_send_to},
 * {@code send_to_without_registered_sender}, {@code remove_session_cleans_up_sender})
 * are not ported: the Java ReplaySessionManager has no per-session sender registry.
 */
class ReplaySessionManagerTest {

	@Test
	void addAndRemoveSession() {
		ReplaySessionManager m = new ReplaySessionManager();
		Session s = mock(Session.class);
		m.addSession(s, "replay1", "coach1");
		assertTrue(m.has(s));
		m.removeSession(s);
		assertFalse(m.has(s));
	}

	@Test
	void firstSessionGetsControl() {
		ReplaySessionManager m = new ReplaySessionManager();
		Session s = mock(Session.class);
		m.addSession(s, "r", "coach1");
		assertTrue(m.hasControl(s));
	}

	@Test
	void secondSessionNoControl() {
		ReplaySessionManager m = new ReplaySessionManager();
		Session s1 = mock(Session.class);
		Session s2 = mock(Session.class);
		m.addSession(s1, "r", "coach1");
		m.addSession(s2, "r", "coach2");
		assertFalse(m.hasControl(s2));
	}

	@Test
	void getLastPingDefaultZero() {
		ReplaySessionManager m = new ReplaySessionManager();
		assertEquals(0, m.getLastPing(mock(Session.class)));
	}

	@Test
	void setAndGetLastPing() {
		ReplaySessionManager m = new ReplaySessionManager();
		Session s = mock(Session.class);
		m.addSession(s, "r", "c");
		m.setLastPing(s, 12345);
		assertEquals(12345, m.getLastPing(s));
	}

	@Test
	void transferControl() {
		ReplaySessionManager m = new ReplaySessionManager();
		Session s1 = mock(Session.class);
		Session s2 = mock(Session.class);
		m.addSession(s1, "r", "coach1");
		m.addSession(s2, "r", "coach2");
		assertTrue(m.transferControl(s1, "coach2"));
		assertFalse(m.hasControl(s1));
		assertTrue(m.hasControl(s2));
	}
}
