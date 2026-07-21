package com.fumbbl.ffb.client.report.bb2020;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.mixed.ReportPrayerAmount;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class PrayerAmountMessageTest extends ReportMessageTestBase {

	@BeforeEach
	public void setUpTeams() {
		given(game.getTeamHome().getName()).willReturn("Team home");
		given(game.getTeamAway().getName()).willReturn("Team away");
	}

	@Test
	public void rendersTvTextForBothTeams() {
		ReportPrayerAmount report = new ReportPrayerAmount(1000, 2000, 1, true);
		List<Run> runs = render(new PrayerAmountMessage(), report);
		assertTrue(runs.stream().anyMatch(r -> " has a TV of 1,000 after buying inducements.".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> " has a TV of 2,000 after buying inducements.".equals(r.text)));
	}

	@Test
	public void singularPrayerTextForAmountOne() {
		ReportPrayerAmount report = new ReportPrayerAmount(500, 500, 1, true);
		List<Run> runs = render(new PrayerAmountMessage(), report);
		assertTrue(runs.stream().anyMatch(r -> " is granted 1 Prayer to Nuffle".equals(r.text)));
	}

	@Test
	public void pluralPrayersTextForAmountOtherThanOne() {
		ReportPrayerAmount report = new ReportPrayerAmount(500, 500, 3, false);
		List<Run> runs = render(new PrayerAmountMessage(), report);
		assertTrue(runs.stream().anyMatch(r -> " is granted 3 Prayers to Nuffle".equals(r.text)));
	}

	@Test
	public void awayTeamReceivesPrayersWhenFlagFalse() {
		ReportPrayerAmount report = new ReportPrayerAmount(500, 500, 1, false);
		List<Run> runs = render(new PrayerAmountMessage(), report);
		assertTrue(runs.stream().anyMatch(r -> r.textStyle == TextStyle.AWAY && "Team away".equals(r.text)));
	}
}
