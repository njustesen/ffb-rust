package com.fumbbl.ffb.inducement.bb2025;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/inducement/bb2025/prayer.rs for {@link Prayer}.
 * The Rust variant BLESSED_STATUE_OF_NUFFLE is named BLESSING_OF_NUFFLE in the Java BB2025 enum (same prayer).
 */
public class PrayerTest {

	@Test
	public void allPrayersHaveNames() {
		Prayer[] prayers = new Prayer[] { Prayer.TREACHEROUS_TRAPDOOR, Prayer.FRIENDS_WITH_THE_REF, Prayer.STILETTO,
			Prayer.IRON_MAN, Prayer.KNUCKLE_DUSTERS, Prayer.BAD_HABITS, Prayer.GREASY_CLEATS,
			Prayer.BLESSING_OF_NUFFLE, Prayer.MOLES_UNDER_THE_PITCH, Prayer.PERFECT_PASSING, Prayer.DAZZLING_CATCHING,
			Prayer.FAN_INTERACTION, Prayer.FOULING_FRENZY, Prayer.THROW_A_ROCK, Prayer.UNDER_SCRUTINY,
			Prayer.INTENSIVE_TRAINING };
		for (Prayer p : prayers) {
			assertFalse(p.getName().isEmpty());
		}
	}
}
