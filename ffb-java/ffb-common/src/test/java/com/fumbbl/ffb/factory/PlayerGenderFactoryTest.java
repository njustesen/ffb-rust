package com.fumbbl.ffb.factory;

import com.fumbbl.ffb.PlayerGender;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/factory/player_gender_factory.rs
 * for {@link PlayerGenderFactory}.
 */
public class PlayerGenderFactoryTest {

	@Test
	public void forNameReturnsKnownGender() {
		assertNotNull(new PlayerGenderFactory().forName("male"));
		assertNotNull(new PlayerGenderFactory().forName("female"));
	}

	@Test
	public void forNameUnknownReturnsNone() {
		assertNull(new PlayerGenderFactory().forName("invalid"));
	}

	@Test
	public void forNameASecondKnownVariant() {
		assertEquals(PlayerGender.NONBINARY, new PlayerGenderFactory().forName("nonbinary"));
	}

	@Test
	public void forNameEmptyStringReturnsNone() {
		assertNull(new PlayerGenderFactory().forName(""));
	}
}
