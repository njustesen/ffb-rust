package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.inducement.Card;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportPlayCard;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class PlayCardMessageTest extends ReportMessageTestBase {

	@Mock
	private Card card;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Test
	public void renderWithoutPlayerPrintsPlayedAndBlankLine() {
		given(card.getName()).willReturn("Bribery");

		ReportPlayCard report = new ReportPlayCard("home", card);
		List<Run> runs = render(new PlayCardMessage(), report);

		assertEquals("Card Bribery is played.", runs.get(0).text);
		assertEquals(TextStyle.BOLD, runs.get(0).textStyle);
		assertEquals(3, runs.size());
		assertEquals(null, runs.get(1).text);
	}

	@Test
	public void renderWithPlayerPrintsPlayerNameAndPeriod() {
		given(card.getName()).willReturn("Bribery");
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Grombrindal");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportPlayCard report = new ReportPlayCard("home", card, "p1");
		List<Run> runs = render(new PlayCardMessage(), report);

		assertEquals("Card Bribery is played on ", runs.get(0).text);
		assertEquals("Grombrindal", runs.get(1).text);
		assertEquals(TextStyle.HOME_BOLD, runs.get(1).textStyle);
		assertEquals(".", runs.get(2).text);
	}

	@Test
	public void renderWithPlayerOnAwayTeamUsesAwayStyle() {
		given(card.getName()).willReturn("Poison");
		given(game.getPlayerById("p2")).willReturn(player);
		given(player.getName()).willReturn("Skitter");
		given(game.getTeamHome().hasPlayer(player)).willReturn(false);

		ReportPlayCard report = new ReportPlayCard("away", card, "p2");
		List<Run> runs = render(new PlayCardMessage(), report);

		assertEquals(TextStyle.AWAY_BOLD, runs.get(1).textStyle);
	}
}
