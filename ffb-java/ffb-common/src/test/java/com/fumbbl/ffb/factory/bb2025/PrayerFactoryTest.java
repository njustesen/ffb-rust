package com.fumbbl.ffb.factory.bb2025;

import com.fumbbl.ffb.factory.IFactorySource;
import com.fumbbl.ffb.inducement.Prayer;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.InducementSet;
import com.fumbbl.ffb.net.NetCommandTestUtil;

import org.junit.jupiter.api.Test;

import java.util.Arrays;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/factory/bb2025/prayer_factory.rs
 * for {@link PrayerFactory}.
 * <p>
 * The {@code value_of} test is not ported: the Java {@code valueOf} throws
 * {@link IllegalArgumentException} for an unknown enum name whereas the Rust variant returns
 * {@code None}.
 */
public class PrayerFactoryTest {

	private static PrayerFactory initialized() {
		IFactorySource app = NetCommandTestUtil.applicationSource();
		Game game = new Game(app, app.getFactoryManager());
		PrayerFactory factory = new PrayerFactory();
		factory.initialize(game);
		return factory;
	}

	@Test
	public void initializeHas16Prayers() {
		PrayerFactory f = initialized();
		assertEquals(16, f.allPrayerRolls().size());
		assertEquals(com.fumbbl.ffb.inducement.bb2025.Prayer.DAZZLING_CATCHING, f.forRoll(11));
	}

	@Test
	public void forNameIsCaseInsensitive() {
		PrayerFactory f = initialized();
		assertEquals(com.fumbbl.ffb.inducement.bb2025.Prayer.DAZZLING_CATCHING, f.forName("dazzling catching"));
	}

	@Test
	public void intensivePrayerIsIntensiveTraining() {
		PrayerFactory f = initialized();
		assertEquals(com.fumbbl.ffb.inducement.bb2025.Prayer.INTENSIVE_TRAINING, f.intensivePrayer());
	}

	@Test
	public void availablePrayerRollsExcludesAlreadyHeldPrayer() {
		PrayerFactory f = initialized();
		InducementSet team = new InducementSet();
		team.addPrayer(f.forName("Iron Man"));
		InducementSet opponent = new InducementSet();
		List<Integer> available = f.availablePrayerRolls(team, opponent);
		assertFalse(available.contains(4));
		assertTrue(available.contains(1));
	}

	@Test
	public void availablePrayerRollsExcludesBothTeamAffectingPrayerHeldByOpponent() {
		PrayerFactory f = initialized();
		InducementSet team = new InducementSet();
		InducementSet opponent = new InducementSet();
		opponent.addPrayer(f.forName("Treacherous Trapdoor"));
		List<Integer> available = f.availablePrayerRolls(team, opponent);
		assertFalse(available.contains(1));
	}

	@Test
	public void sortOrdersByRollAndFiltersToGivenSet() {
		PrayerFactory f = initialized();
		Set<Prayer> unsorted = new HashSet<>();
		unsorted.add(com.fumbbl.ffb.inducement.bb2025.Prayer.INTENSIVE_TRAINING);
		unsorted.add(com.fumbbl.ffb.inducement.bb2025.Prayer.TREACHEROUS_TRAPDOOR);
		List<Prayer> sorted = f.sort(unsorted);
		assertEquals(Arrays.asList(com.fumbbl.ffb.inducement.bb2025.Prayer.TREACHEROUS_TRAPDOOR,
			com.fumbbl.ffb.inducement.bb2025.Prayer.INTENSIVE_TRAINING), sorted);
	}
}
