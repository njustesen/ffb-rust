package com.fumbbl.ffb.inducement.bb2020;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/inducement/bb2020/prayer.rs for {@link Prayer}.
 */
public class PrayerTest {

	@Test
	public void allPrayersHaveNames() {
		Prayer[] prayers = new Prayer[] { Prayer.TREACHEROUS_TRAPDOOR, Prayer.FRIENDS_WITH_THE_REF, Prayer.STILETTO,
			Prayer.IRON_MAN, Prayer.KNUCKLE_DUSTERS, Prayer.BAD_HABITS, Prayer.GREASY_CLEATS,
			Prayer.BLESSED_STATUE_OF_NUFFLE, Prayer.MOLES_UNDER_THE_PITCH, Prayer.PERFECT_PASSING,
			Prayer.FAN_INTERACTION, Prayer.NECESSARY_VIOLENCE, Prayer.FOULING_FRENZY, Prayer.THROW_A_ROCK,
			Prayer.UNDER_SCRUTINY, Prayer.INTENSIVE_TRAINING };
		for (Prayer p : prayers) {
			assertFalse(p.getName().isEmpty());
		}
	}
}
