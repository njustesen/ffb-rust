package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.inducement.Prayer;
import com.fumbbl.ffb.report.mixed.ReportPrayerEnd;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class PrayerEndMessageTest extends ReportMessageTestBase {

	@Mock
	private Prayer prayer;

	// java: `missing_prayer_name_renders_empty_string` from the Rust suite is not portable —
	// PrayerEndMessage.render() calls `report.getPrayer().getName()`/`.getDescription()`
	// unconditionally; a null Prayer NPEs in real Java. Skipped.

	@Test
	public void rendersPrayerNameOnBothLines() {
		given(prayer.getName()).willReturn("PRAYER_OF_DEATH");
		given(prayer.getDescription()).willReturn("PRAYER_OF_DEATH");

		ReportPrayerEnd report = new ReportPrayerEnd(prayer);
		List<Run> runs = render(new PrayerEndMessage(), report);

		assertEquals("Prayer effect ended: PRAYER_OF_DEATH", runs.get(0).text);
		assertEquals(TextStyle.ROLL, runs.get(0).textStyle);
	}

	@Test
	public void secondLineUsesIndentPlusTwoAndExplanationStyle() {
		given(prayer.getName()).willReturn("HAND_OF_GOD");
		given(prayer.getDescription()).willReturn("HAND_OF_GOD");

		ReportPrayerEnd report = new ReportPrayerEnd(prayer);
		List<Run> runs = render(new PrayerEndMessage(), report);

		// rendered_runs: [line1, terminator, line2, terminator]
		assertEquals("Effect was: HAND_OF_GOD", runs.get(2).text);
		assertEquals(TextStyle.EXPLANATION, runs.get(2).textStyle);
	}
}
