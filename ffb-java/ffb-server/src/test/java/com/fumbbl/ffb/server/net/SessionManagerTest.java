package com.fumbbl.ffb.server.net;

import com.fumbbl.ffb.ClientMode;
import org.eclipse.jetty.websocket.api.Session;
import org.junit.jupiter.api.Test;

import java.util.Arrays;
import java.util.Collections;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertSame;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.mock;

/**
 * Mirrors the Rust {@code session_manager} tests. Rust identifies sessions by an
 * i64 handle; Java keys on the Jetty {@link Session} object, so distinct mocks
 * stand in for the handles.
 *
 * <p>The Rust {@code sender_for_*} and {@code send_all_*} tests are not ported:
 * Rust registers an mpsc sender per session, whereas the Java SessionManager
 * has no sender registry (messages are written through {@code Session.getRemote()}),
 * so there is nothing analogous to assert without a live socket.
 */
class SessionManagerTest {

	private static List<String> noProperties() {
		return Collections.emptyList();
	}

	@Test
	void addAndLookupSession() {
		SessionManager sm = new SessionManager();
		Session s = mock(Session.class);
		sm.addSession(s, 100, "Home", ClientMode.PLAYER, true, noProperties());
		assertEquals(100, sm.getGameIdForSession(s));
		assertEquals("Home", sm.getCoachForSession(s));
		assertEquals(ClientMode.PLAYER, sm.getModeForSession(s));
	}

	@Test
	void homeAndAwayLookup() {
		SessionManager sm = new SessionManager();
		Session home = mock(Session.class);
		Session away = mock(Session.class);
		sm.addSession(home, 100, "Home", ClientMode.PLAYER, true, noProperties());
		sm.addSession(away, 100, "Away", ClientMode.PLAYER, false, noProperties());
		assertSame(home, sm.getSessionOfHomeCoach(100));
		assertSame(away, sm.getSessionOfAwayCoach(100));
	}

	@Test
	void removeSessionCleansUp() {
		SessionManager sm = new SessionManager();
		Session s = mock(Session.class);
		sm.addSession(s, 100, "Coach", ClientMode.PLAYER, true, noProperties());
		sm.removeSession(s);
		assertEquals(0, sm.getGameIdForSession(s));
		assertEquals(0, sm.getSessionsForGameId(100).length);
	}

	@Test
	void sessionsWithoutAwayExcludesAway() {
		SessionManager sm = new SessionManager();
		Session home = mock(Session.class);
		Session away = mock(Session.class);
		Session spec = mock(Session.class);
		sm.addSession(home, 100, "Home", ClientMode.PLAYER, true, noProperties());
		sm.addSession(away, 100, "Away", ClientMode.PLAYER, false, noProperties());
		sm.addSession(spec, 100, "Spec", ClientMode.SPECTATOR, false, noProperties());
		List<Session> withoutAway = Arrays.asList(sm.getSessionsWithoutAwayCoach(100));
		assertTrue(withoutAway.contains(home));
		assertTrue(withoutAway.contains(spec));
		assertFalse(withoutAway.contains(away));
	}
}
