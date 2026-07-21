package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportHandOver;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class HandOverMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player thrower;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player catcher;

	private List<String> texts(List<Run> runs) {
		List<String> texts = new ArrayList<>();
		for (Run run : runs) {
			if (run.text != null) {
				texts.add(run.text);
			}
		}
		return texts;
	}

	private void stubPlayers() {
		given(game.getActingPlayer().getPlayer()).willReturn(thrower);
		given(thrower.getName()).willReturn("thrower");
		given(game.getPlayerById("catcher")).willReturn(catcher);
		given(catcher.getName()).willReturn("catcher");
		given(game.getTeamHome().hasPlayer(thrower)).willReturn(true);
		given(game.getTeamHome().hasPlayer(catcher)).willReturn(false);
	}

	@Test
	public void rendersThrowerHandsOverToCatcher() {
		stubPlayers();

		ReportHandOver report = new ReportHandOver("catcher");
		List<Run> runs = render(new HandOverMessage(), report);

		assertEquals(Arrays.asList("thrower", " hands over the ball to ", "catcher", ":"), texts(runs));
	}

	@Test
	public void usesBoldTextStyleForVerbAndColon() {
		stubPlayers();

		ReportHandOver report = new ReportHandOver("catcher");
		List<Run> runs = render(new HandOverMessage(), report);

		assertEquals(TextStyle.BOLD, runs.get(1).textStyle);
		assertEquals(TextStyle.BOLD, runs.get(3).textStyle);
	}

	@Test
	public void throwerAndCatcherUseHomeAwayBoldStyles() {
		stubPlayers();

		ReportHandOver report = new ReportHandOver("catcher");
		List<Run> runs = render(new HandOverMessage(), report);

		assertEquals(TextStyle.HOME_BOLD, runs.get(0).textStyle);
		assertEquals(TextStyle.AWAY_BOLD, runs.get(2).textStyle);
	}
}
