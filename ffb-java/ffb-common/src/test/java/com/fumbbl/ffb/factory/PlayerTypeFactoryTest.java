package com.fumbbl.ffb.factory;

import com.fumbbl.ffb.PlayerType;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/factory/player_type_factory.rs
 * for {@link PlayerTypeFactory}.
 */
public class PlayerTypeFactoryTest {

	@Test
	public void forNameReturnsKnownType() {
		assertEquals(PlayerType.REGULAR, new PlayerTypeFactory().forName("Regular"));
	}

	@Test
	public void forNameUnknownReturnsNone() {
		assertNull(new PlayerTypeFactory().forName("invalid"));
	}

	@Test
	public void forNameASecondKnownVariant() {
		assertEquals(PlayerType.STAR, new PlayerTypeFactory().forName("Star"));
	}

	@Test
	public void forNameEmptyStringReturnsNone() {
		assertNull(new PlayerTypeFactory().forName(""));
	}
}
