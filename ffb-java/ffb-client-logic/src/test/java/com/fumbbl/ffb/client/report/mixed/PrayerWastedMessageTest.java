package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.mixed.ReportPrayerWasted;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class PrayerWastedMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Test
	public void noPlayerIdUsesNoEligiblePlayersMessage() {
		ReportPrayerWasted report = new ReportPrayerWasted("PRAYER_OF_DEATH", null);
		List<Run> runs = render(new PrayerWastedMessage(), report);

		assertEquals("Prayer PRAYER_OF_DEATH is wasted since there are no eligible players.", runs.get(0).text);
		assertEquals(2, runs.size());
	}

	@Test
	public void emptyPlayerIdTreatedAsNotProvided() {
		ReportPrayerWasted report = new ReportPrayerWasted("PRAYER_OF_DEATH", "");
		List<Run> runs = render(new PrayerWastedMessage(), report);

		assertEquals("Prayer PRAYER_OF_DEATH is wasted since there are no eligible players.", runs.get(0).text);
	}

	@Test
	public void providedPlayerIdPrintsPlayerAndSelectedMessage() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Joe");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportPrayerWasted report = new ReportPrayerWasted("HAND_OF_GOD", "p1");
		List<Run> runs = render(new PrayerWastedMessage(), report);

		assertEquals("Prayer HAND_OF_GOD is wasted since there are no eligible skills.", runs.get(0).text);
		Run playerRun = runs.stream().filter(r -> "Joe".equals(r.text)).findFirst().orElseThrow();
		assertEquals(TextStyle.HOME_BOLD, playerRun.textStyle);
		// runs.get(runs.size() - 1) is the null terminator appended by the trailing
		// println(...); the real sentence is the last non-null run before it.
		Run last = runs.stream().filter(r -> r.text != null).reduce((a, b) -> b).orElseThrow();
		assertEquals(" was the selected player", last.text);
	}
}
