package com.fumbbl.ffb.inducement.bb2025;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/inducement/bb2025/prayers.rs for {@link Prayers}.
 * Rust's get_prayer(roll) maps to the combined all-prayers map; the roll-8 prayer is BLESSING_OF_NUFFLE in Java.
 */
public class PrayersTest {

	@Test
	public void testAllPrayersHas16Entries() {
		Prayers prayers = new Prayers();
		assertEquals(16, prayers.getAllPrayers().size());
	}

	@Test
	public void testRoll8IsBlessingOfNuffle() {
		Prayers prayers = new Prayers();
		assertEquals(Prayer.BLESSING_OF_NUFFLE, prayers.getAllPrayers().get(8));
	}
}
