package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.report.mixed.ReportPrayerAmount;
import org.junit.jupiter.api.Test;

import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class PrayerAmountMessageTest extends ReportMessageTestBase {

	private void stubTeamNames() {
		given(game.getTeamHome().getName()).willReturn("Team home");
		given(game.getTeamAway().getName()).willReturn("Team away");
	}

	@Test
	public void homeTeamReceivesPrayersUsesHomeStyle() {
		stubTeamNames();
		ReportPrayerAmount report = new ReportPrayerAmount(1_000_000, 900_000, 2, true);
		List<Run> runs = render(new PrayerAmountMessage(), report);
		Run homeRun = runs.stream().filter(r -> "Team home".equals(r.text)).findFirst().orElseThrow();
		assertEquals(TextStyle.HOME_BOLD, homeRun.textStyle);
	}

	@Test
	public void tvTextFormatsThousands() {
		stubTeamNames();
		ReportPrayerAmount report = new ReportPrayerAmount(1_000_000, 900_000, 2, true);
		List<Run> runs = render(new PrayerAmountMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).filter(Objects::nonNull).collect(Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> t.contains("1,000,000")));
		assertTrue(texts.stream().anyMatch(t -> t.contains("900,000")));
	}

	@Test
	public void singularPrayerWhenAmountIsOne() {
		stubTeamNames();
		ReportPrayerAmount report = new ReportPrayerAmount(0, 0, 1, false);
		List<Run> runs = render(new PrayerAmountMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).filter(Objects::nonNull).collect(Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> t.equals(" is granted 1 additional Prayer to Nuffle")));
	}

	@Test
	public void pluralPrayersWhenAmountIsNotOne() {
		stubTeamNames();
		ReportPrayerAmount report = new ReportPrayerAmount(0, 0, 3, false);
		List<Run> runs = render(new PrayerAmountMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).filter(Objects::nonNull).collect(Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> t.equals(" is granted 3 additional Prayers to Nuffle")));

		List<Run> reversed = new java.util.ArrayList<>(runs);
		java.util.Collections.reverse(reversed);
		Run awayRun = reversed.stream().filter(r -> "Team away".equals(r.text)).findFirst().orElseThrow();
		assertEquals(TextStyle.AWAY, awayRun.textStyle);
	}

	@Test
	public void reportIdIsPrayerAmount() {
		assertEquals(ReportId.PRAYER_AMOUNT.getKey(), new PrayerAmountMessage().getKey());
	}
}
