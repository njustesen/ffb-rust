package com.fumbbl.ffb.factory.bb2020;

import com.fumbbl.ffb.factory.IFactorySource;
import com.fumbbl.ffb.inducement.Prayer;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.InducementSet;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import com.fumbbl.ffb.option.GameOptionBoolean;
import com.fumbbl.ffb.option.GameOptionId;

import org.junit.jupiter.api.Test;

import java.util.Arrays;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/factory/bb2020/prayer_factory.rs
 * for {@link PrayerFactory}.
 * <p>
 * The {@code value_of} test is not ported: the Java {@code valueOf} throws
 * {@link IllegalArgumentException} for an unknown enum name whereas the Rust variant returns
 * {@code None}.
 */
public class PrayerFactoryTest {

	private static PrayerFactory initialized(boolean useLeagueTable) {
		IFactorySource app = NetCommandTestUtil.applicationSource();
		Game game = new Game(app, app.getFactoryManager());
		GameOptionBoolean leagueTable = new GameOptionBoolean(GameOptionId.INDUCEMENT_PRAYERS_USE_LEAGUE_TABLE);
		leagueTable.setValue(useLeagueTable);
		game.getOptions().addOption(leagueTable);
		PrayerFactory factory = new PrayerFactory();
		factory.initialize(game);
		return factory;
	}

	@Test
	public void initializeWithoutLeagueTableHas8ExhibitionPrayers() {
		PrayerFactory f = initialized(false);
		assertEquals(8, f.allPrayerRolls().size());
		assertNull(f.forRoll(9));
	}

	@Test
	public void initializeWithLeagueTableHas16Prayers() {
		PrayerFactory f = initialized(true);
		assertEquals(16, f.allPrayerRolls().size());
		assertEquals(com.fumbbl.ffb.inducement.bb2020.Prayer.INTENSIVE_TRAINING, f.forRoll(16));
	}

	@Test
	public void forNameIsCaseInsensitive() {
		PrayerFactory f = initialized(false);
		assertEquals(com.fumbbl.ffb.inducement.bb2020.Prayer.IRON_MAN, f.forName("iron man"));
		assertEquals(com.fumbbl.ffb.inducement.bb2020.Prayer.IRON_MAN, f.forName("Iron Man"));
	}

	@Test
	public void intensivePrayerIsIntensiveTraining() {
		PrayerFactory f = initialized(true);
		assertEquals(com.fumbbl.ffb.inducement.bb2020.Prayer.INTENSIVE_TRAINING, f.intensivePrayer());
	}

	@Test
	public void availablePrayerRollsExcludesAlreadyHeldPrayer() {
		PrayerFactory f = initialized(false);
		InducementSet team = new InducementSet();
		team.addPrayer(f.forName("Iron Man"));
		InducementSet opponent = new InducementSet();
		List<Integer> available = f.availablePrayerRolls(team, opponent);
		assertFalse(available.contains(4)); // roll 4 == IRON_MAN
		assertTrue(available.contains(1)); // roll 1 == TREACHEROUS_TRAPDOOR, still available
	}

	@Test
	public void availablePrayerRollsExcludesBothTeamAffectingPrayerHeldByOpponent() {
		PrayerFactory f = initialized(false);
		InducementSet team = new InducementSet();
		InducementSet opponent = new InducementSet();
		opponent.addPrayer(f.forName("Treacherous Trapdoor"));
		List<Integer> available = f.availablePrayerRolls(team, opponent);
		// roll 1 == TREACHEROUS_TRAPDOOR, affects both teams, held by opponent -> excluded.
		assertFalse(available.contains(1));
	}

	@Test
	public void sortOrdersByRollAndFiltersToGivenSet() {
		PrayerFactory f = initialized(false);
		Set<Prayer> unsorted = new HashSet<>();
		unsorted.add(com.fumbbl.ffb.inducement.bb2020.Prayer.BLESSED_STATUE_OF_NUFFLE);
		unsorted.add(com.fumbbl.ffb.inducement.bb2020.Prayer.TREACHEROUS_TRAPDOOR);
		List<Prayer> sorted = f.sort(unsorted);
		assertEquals(Arrays.asList(com.fumbbl.ffb.inducement.bb2020.Prayer.TREACHEROUS_TRAPDOOR,
			com.fumbbl.ffb.inducement.bb2020.Prayer.BLESSED_STATUE_OF_NUFFLE), sorted);
	}
}
