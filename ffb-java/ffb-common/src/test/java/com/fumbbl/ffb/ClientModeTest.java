package com.fumbbl.ffb;

import com.fumbbl.ffb.factory.ClientModeFactory;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/client_mode.rs for {@link ClientMode}.
 */
public class ClientModeTest {

	private final ClientModeFactory factory = new ClientModeFactory();

	@Test
	public void forNamePlayer() {
		assertEquals(ClientMode.PLAYER, factory.forName("player"));
	}

	@Test
	public void forNameUnknownReturnsNone() {
		assertNull(factory.forName("admin"));
	}

	@Test
	public void getNameRoundTrip() {
		for (ClientMode mode : new ClientMode[]{ClientMode.PLAYER, ClientMode.SPECTATOR, ClientMode.REPLAY}) {
			assertEquals(mode, factory.forName(mode.getName()));
		}
	}

	@Test
	public void forNameSpectatorAndReplay() {
		assertEquals(ClientMode.SPECTATOR, factory.forName("spectator"));
		assertEquals(ClientMode.REPLAY, factory.forName("replay"));
	}

}
