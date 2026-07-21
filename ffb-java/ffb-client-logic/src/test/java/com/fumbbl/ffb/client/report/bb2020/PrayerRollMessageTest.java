package com.fumbbl.ffb.client.report.bb2020;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.factory.PrayerFactory;
import com.fumbbl.ffb.inducement.bb2020.Prayer;
import com.fumbbl.ffb.report.bb2020.ReportPrayerRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

// Note: out-of-range-roll case from the Rust test suite
// (`out_of_range_roll_only_renders_roll_line`) is intentionally NOT ported. The Rust
// translation added a defensive `if let Some(prayer) = prayer` null-check that has no
// counterpart in the real `PrayerRollMessage.java`, which calls `prayer.getName()`
// unconditionally and would throw a NullPointerException for a roll with no matching
// Prayer. Porting that sub-case here would misrepresent the actual Java behavior.
class PrayerRollMessageTest extends ReportMessageTestBase {

	@Mock
	private PrayerFactory prayerFactory;

	@Test
	public void rendersRollLineAndPrayerDetailsForTreacherousTrapdoor() {
		org.mockito.Mockito.doReturn(prayerFactory).when(game).getFactory(FactoryType.Factory.PRAYER);
		given(prayerFactory.forRoll(1)).willReturn(Prayer.TREACHEROUS_TRAPDOOR);
		ReportPrayerRoll report = new ReportPrayerRoll(1);
		List<Run> runs = render(new PrayerRollMessage(), report);

		assertEquals(6, runs.size());
		assertEquals("Prayer Roll [ 1 ]", runs.get(0).text);
		assertEquals(TextStyle.ROLL, runs.get(0).textStyle);
		assertEquals("Treacherous Trapdoor", runs.get(2).text);
		assertEquals(TextStyle.BOLD, runs.get(2).textStyle);
		assertEquals(
			"For this half: Trapdoors appear. On a roll of 1 a player stepping on them falls through them",
			runs.get(4).text);
		assertEquals(TextStyle.EXPLANATION, runs.get(4).textStyle);
	}

	@Test
	public void rendersPerfectPassingForRoll10() {
		org.mockito.Mockito.doReturn(prayerFactory).when(game).getFactory(FactoryType.Factory.PRAYER);
		given(prayerFactory.forRoll(10)).willReturn(Prayer.PERFECT_PASSING);
		ReportPrayerRoll report = new ReportPrayerRoll(10);
		List<Run> runs = render(new PrayerRollMessage(), report);

		assertEquals("Perfect Passing", runs.get(2).text);
		assertEquals(
			"For the entire game: Completions generate 2 instead of 1 spp",
			runs.get(4).text);
	}
}
