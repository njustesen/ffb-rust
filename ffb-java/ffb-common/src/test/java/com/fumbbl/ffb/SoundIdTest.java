package com.fumbbl.ffb;

import com.fumbbl.ffb.factory.SoundIdFactory;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/sound_id.rs for {@link SoundId}.
 */
public class SoundIdTest {

	private final SoundIdFactory factory = new SoundIdFactory();

	@Test
	public void forNameCaseInsensitive() {
		assertEquals(SoundId.BLOCK, factory.forName("block"));
		assertEquals(SoundId.BLOCK, factory.forName("BLOCK"));
		assertNull(factory.forName("invalid"));
	}

	@Test
	public void forNameRoundTripsAllVariants() {
		for (SoundId v : SoundId.values()) {
			assertEquals(v, factory.forName(v.getName()), "round trip failed for " + v);
		}
	}

	@Test
	public void allVariantsHaveNonEmptyNames() {
		for (SoundId v : SoundId.values()) {
			assertFalse(v.getName().isEmpty(), v + " has empty name");
		}
	}

}
