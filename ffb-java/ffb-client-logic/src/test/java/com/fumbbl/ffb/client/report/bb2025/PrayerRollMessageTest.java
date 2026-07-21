package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.factory.PrayerFactory;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.report.bb2025.ReportPrayerRoll;
import org.junit.jupiter.api.Test;

import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class PrayerRollMessageTest extends ReportMessageTestBase {

	private void stubPrayerFactory() {
		PrayerFactory factory = new com.fumbbl.ffb.factory.bb2025.PrayerFactory();
		factory.initialize(null);
		// game is a RETURNS_DEEP_STUBS mock; getFactory(...) has a bound generic return type
		// (T extends INamedObjectFactory<?>), so given(game.<PrayerFactory>getFactory(...))
		// forces a checkcast to PrayerFactory against the deep-stub's default proxy (which only
		// implements INamedObjectFactory) BEFORE the stubbing is even recorded, causing a
		// ClassCastException. doReturn(...).when(...) avoids evaluating that generic checkcast.
		org.mockito.Mockito.doReturn(factory).when(game).getFactory(FactoryType.Factory.PRAYER);
	}

	@Test
	public void homeTeamUsesHomeBoldStyle() {
		stubPrayerFactory();
		ReportPrayerRoll report = new ReportPrayerRoll("Home Ultras", 8, true);
		List<Run> runs = render(new PrayerRollMessage(), report);
		Run teamRun = runs.stream().filter(r -> "Home Ultras".equals(r.text)).findFirst().orElseThrow();
		assertEquals(TextStyle.HOME_BOLD, teamRun.textStyle);
	}

	@Test
	public void awayTeamUsesAwayBoldStyle() {
		stubPrayerFactory();
		ReportPrayerRoll report = new ReportPrayerRoll("Away Raiders", 3, false);
		List<Run> runs = render(new PrayerRollMessage(), report);
		Run teamRun = runs.stream().filter(r -> "Away Raiders".equals(r.text)).findFirst().orElseThrow();
		assertEquals(TextStyle.AWAY_BOLD, teamRun.textStyle);
	}

	@Test
	public void rollOf8ResolvesToBlessingOfNuffle() {
		// Note: the Rust test for this roll expects "Blessed Statue of Nuffle", but the real
		// Java bb2025 prayer table (Prayers.java) maps roll 8 to Prayer.BLESSING_OF_NUFFLE
		// ("Blessing of Nuffle"); the Rust data has diverged from the Java source of truth.
		// Asserting against the actual Java content here.
		stubPrayerFactory();
		ReportPrayerRoll report = new ReportPrayerRoll("Home Ultras", 8, true);
		List<Run> runs = render(new PrayerRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).filter(Objects::nonNull).collect(Collectors.toList());
		assertTrue(texts.contains("Blessing of Nuffle"));
	}

	@Test
	public void rollOf16ResolvesToIntensiveTraining() {
		stubPrayerFactory();
		ReportPrayerRoll report = new ReportPrayerRoll("Home Ultras", 16, true);
		List<Run> runs = render(new PrayerRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).filter(Objects::nonNull).collect(Collectors.toList());
		assertTrue(texts.contains("Intensive Training"));
	}

	@Test
	public void reportIdIsPrayerRoll() {
		assertEquals(ReportId.PRAYER_ROLL.getKey(), new PrayerRollMessage().getKey());
	}

	@Test
	public void prayerDescriptionIsRenderedAlongsideDuration() {
		// java: `println(getIndent() + 2, TextStyle.EXPLANATION,
		// prayer.getDuration().getDescription() + ": " + prayer.getDescription());` — the
		// prayer's actual rules text must be printed, not just the duration label.
		stubPrayerFactory();
		ReportPrayerRoll report = new ReportPrayerRoll("Home Ultras", 2, true);
		List<Run> runs = render(new PrayerRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).filter(Objects::nonNull).collect(Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> t.contains("Argue the call succeeds on 5+")));
	}
}
