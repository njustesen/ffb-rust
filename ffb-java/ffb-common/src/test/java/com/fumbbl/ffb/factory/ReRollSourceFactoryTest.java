package com.fumbbl.ffb.factory;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/factory/re_roll_source_factory.rs
 * for {@link ReRollSourceFactory}.
 */
public class ReRollSourceFactoryTest {

	@Test
	public void forNameReturnsKnownSource() {
		ReRollSourceFactory f = new ReRollSourceFactory();
		assertNotNull(f.forName("Pro"));
	}

	@Test
	public void forNameUnknownReturnsNone() {
		ReRollSourceFactory f = new ReRollSourceFactory();
		assertNull(f.forName("NOT_VALID"));
	}

	@Test
	public void forNameASecondKnownVariant() {
		ReRollSourceFactory f = new ReRollSourceFactory();
		assertNotNull(f.forName("Team ReRoll"));
	}

	@Test
	public void forNameEmptyStringReturnsNone() {
		ReRollSourceFactory f = new ReRollSourceFactory();
		assertNull(f.forName(""));
	}
}
