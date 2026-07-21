package com.fumbbl.ffb.factory;

import com.fumbbl.ffb.SoundId;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/factory/sound_id_factory.rs
 * for {@link SoundIdFactory}.
 */
public class SoundIdFactoryTest {

	@Test
	public void forNameReturnsKnownSoundId() {
		SoundIdFactory f = new SoundIdFactory();
		assertEquals(SoundId.BLOCK, f.forName("block"));
	}

	@Test
	public void forNameUnknownReturnsNone() {
		assertNull(new SoundIdFactory().forName("invalid"));
	}

	@Test
	public void forNameTouchdownReturnsSome() {
		SoundIdFactory f = new SoundIdFactory();
		assertNotNull(f.forName("touchdown"));
	}

	@Test
	public void forNameEmptyStringReturnsNone() {
		assertNull(new SoundIdFactory().forName(""));
	}
}
