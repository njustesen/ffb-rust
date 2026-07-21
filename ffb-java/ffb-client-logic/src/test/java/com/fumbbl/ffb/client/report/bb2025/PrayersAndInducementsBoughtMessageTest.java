package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.report.bb2025.ReportPrayersAndInducementsBought;
import org.junit.jupiter.api.Test;

import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class PrayersAndInducementsBoughtMessageTest extends ReportMessageTestBase {

	private void stubTeamIds() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamAway().getId()).willReturn("away");
	}

	@Test
	public void firstCallPrintsBuyInducementsHeaderOnce() {
		stubTeamIds();
		ReportPrayersAndInducementsBought report = new ReportPrayersAndInducementsBought("home", 0, 0, 0, 0, 0);
		List<Run> runs = render(new PrayersAndInducementsBoughtMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).filter(Objects::nonNull).collect(Collectors.toList());
		assertTrue(texts.contains("Buy Inducements"));
		assertTrue(statusReport.inducementsBoughtReportReceived);
	}

	@Test
	public void secondCallDoesNotReprintHeader() {
		stubTeamIds();
		statusReport.inducementsBoughtReportReceived = true;
		ReportPrayersAndInducementsBought report = new ReportPrayersAndInducementsBought("home", 0, 0, 0, 0, 0);
		List<Run> runs = render(new PrayersAndInducementsBoughtMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).filter(Objects::nonNull).collect(Collectors.toList());
		assertFalse(texts.contains("Buy Inducements"));
	}

	@Test
	public void noItemsBoughtPrintsNoInducements() {
		stubTeamIds();
		ReportPrayersAndInducementsBought report = new ReportPrayersAndInducementsBought("home", 0, 0, 0, 0, 0);
		List<Run> runs = render(new PrayersAndInducementsBoughtMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).filter(Objects::nonNull).collect(Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> t.equals(" buys no Inducements.")));
	}

	@Test
	public void multipleItemsUseEnumerationAndGoldFormatting() {
		stubTeamIds();
		ReportPrayersAndInducementsBought report = new ReportPrayersAndInducementsBought("away", 2, 1, 0, 150000, 1100000);
		List<Run> runs = render(new PrayersAndInducementsBoughtMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).filter(Objects::nonNull).collect(Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> t.contains("2 Inducements and 1 Star")));
		assertTrue(texts.stream().anyMatch(t -> t.contains("150,000 gold")));
		assertTrue(texts.stream().anyMatch(t -> t.contains("1,100,000")));
	}

	@Test
	public void singleItemSingularWording() {
		stubTeamIds();
		ReportPrayersAndInducementsBought report = new ReportPrayersAndInducementsBought("home", 1, 0, 1, 50000, 900000);
		List<Run> runs = render(new PrayersAndInducementsBoughtMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).filter(Objects::nonNull).collect(Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> t.contains("1 Inducement and 1 Mercenary")));
	}

	@Test
	public void reportIdIsPrayersAndInducementsBought() {
		assertEquals(ReportId.PRAYERS_AND_INDUCEMENTS_BOUGHT.getKey(), new PrayersAndInducementsBoughtMessage().getKey());
	}
}
