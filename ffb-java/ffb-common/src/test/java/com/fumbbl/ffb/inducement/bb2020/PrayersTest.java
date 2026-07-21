package com.fumbbl.ffb.inducement.bb2020;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/inducement/bb2020/prayers.rs for {@link Prayers}.
 * Rust's get_prayer(roll) maps to the exhibition/league prayer maps; roll 1 lives in the exhibition map.
 */
public class PrayersTest {

	@Test
	public void testExhibitionHas8Entries() {
		Prayers prayers = new Prayers();
		assertEquals(8, prayers.getExhibitionPrayers().size());
	}

	@Test
	public void testRoll1IsTreacherousTrapdoor() {
		Prayers prayers = new Prayers();
		assertEquals(Prayer.TREACHEROUS_TRAPDOOR, prayers.getExhibitionPrayers().get(1));
	}
}
