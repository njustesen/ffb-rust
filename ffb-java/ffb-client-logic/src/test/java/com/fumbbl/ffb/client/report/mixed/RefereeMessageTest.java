package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.mixed.ReportReferee;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;
import java.util.stream.Collectors;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class RefereeMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Test
	public void bannedNotUnderScrutiny() {
		given(game.getActingPlayer().getPlayer()).willReturn(player);
		given(player.getName()).willReturn("Joe");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportReferee report = new ReportReferee(true, false);
		List<Run> runs = render(new RefereeMessage(), report);

		// filter out the null-text terminator run appended by the trailing println(...)
		List<String> texts = runs.stream().map(r -> r.text).filter(java.util.Objects::nonNull).collect(Collectors.toList());
		assertEquals(
			List.of("The referee spots the foul ", "and bans ", "Joe", " from the game."),
			texts
		);
	}

	@Test
	public void bannedUnderScrutiny() {
		given(game.getActingPlayer().getPlayer()).willReturn(player);
		given(player.getName()).willReturn("Joe");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportReferee report = new ReportReferee(true, true);
		List<Run> runs = render(new RefereeMessage(), report);

		// filter out the null-text terminator run appended by the trailing println(...)
		List<String> texts = runs.stream().map(r -> r.text).filter(java.util.Objects::nonNull).collect(Collectors.toList());
		assertEquals(
			List.of(
				"The referee spots the foul ",
				"because the team is under scrutiny ",
				"and bans ",
				"Joe",
				" from the game."
			),
			texts
		);
	}

	@Test
	public void notBanned() {
		ReportReferee report = new ReportReferee(false, false);
		List<Run> runs = render(new RefereeMessage(), report);

		assertEquals("The referee didn't spot the foul.", runs.get(0).text);
	}
}
