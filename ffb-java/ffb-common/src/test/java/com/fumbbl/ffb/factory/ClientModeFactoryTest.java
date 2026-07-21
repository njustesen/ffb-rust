package com.fumbbl.ffb.factory;

import com.fumbbl.ffb.ClientMode;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/factory/client_mode_factory.rs
 * for {@link ClientModeFactory}.
 */
public class ClientModeFactoryTest {

	@Test
	public void forNameReturnsKnownMode() {
		assertEquals(ClientMode.PLAYER, new ClientModeFactory().forName("player"));
		assertEquals(ClientMode.SPECTATOR, new ClientModeFactory().forName("spectator"));
	}

	@Test
	public void forNameUnknownReturnsNone() {
		assertNull(new ClientModeFactory().forName("invalid"));
	}

	@Test
	public void forNameASecondKnownVariant() {
		assertEquals(ClientMode.REPLAY, new ClientModeFactory().forName("replay"));
	}

	@Test
	public void forNameEmptyStringReturnsNone() {
		assertNull(new ClientModeFactory().forName(""));
	}

	@Test
	public void forArgumentReturnsKnownMode() {
		assertEquals(ClientMode.PLAYER, new ClientModeFactory().forArgument("-player"));
		assertEquals(ClientMode.SPECTATOR, new ClientModeFactory().forArgument("-spectator"));
		assertEquals(ClientMode.REPLAY, new ClientModeFactory().forArgument("-replay"));
	}

	@Test
	public void forArgumentIsCaseInsensitive() {
		assertEquals(ClientMode.PLAYER, new ClientModeFactory().forArgument("-PLAYER"));
	}

	@Test
	public void forArgumentUnknownReturnsNone() {
		assertNull(new ClientModeFactory().forArgument("-bogus"));
	}
}
