package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.mixed.ReportSelectBlitzTarget;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;
import java.util.stream.Collectors;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class SelectBlitzTargetMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player attacker;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player defender;

	@Test
	public void homeAttackerTargetsAwayDefender() {
		given(game.getPlayerById("p1")).willReturn(attacker);
		given(attacker.getName()).willReturn("Joe");
		given(game.getTeamHome().hasPlayer(attacker)).willReturn(true);
		given(game.getPlayerById("p2")).willReturn(defender);
		given(defender.getName()).willReturn("Jane");
		given(game.getTeamHome().hasPlayer(defender)).willReturn(false);

		ReportSelectBlitzTarget report = new ReportSelectBlitzTarget("p1", "p2");
		List<Run> runs = render(new SelectBlitzTargetMessage(), report);

		// filter out the null-text terminator run appended by the trailing println(...)
		List<String> texts = runs.stream().map(r -> r.text).filter(java.util.Objects::nonNull).collect(Collectors.toList());
		assertEquals(List.of("Joe", " targets ", "Jane", "."), texts);
		assertEquals(TextStyle.HOME, runs.get(0).textStyle);
	}

	@Test
	public void awayAttackerTargetsHomeDefender() {
		given(game.getPlayerById("p2")).willReturn(attacker);
		given(attacker.getName()).willReturn("Jane");
		given(game.getTeamHome().hasPlayer(attacker)).willReturn(false);
		given(game.getPlayerById("p1")).willReturn(defender);
		given(defender.getName()).willReturn("Joe");
		given(game.getTeamHome().hasPlayer(defender)).willReturn(true);

		ReportSelectBlitzTarget report = new ReportSelectBlitzTarget("p2", "p1");
		List<Run> runs = render(new SelectBlitzTargetMessage(), report);

		assertEquals(TextStyle.AWAY, runs.get(0).textStyle);
	}

	@Test
	public void defenderStyleResolvedIndependently() {
		given(game.getPlayerById("p1")).willReturn(attacker);
		given(attacker.getName()).willReturn("Joe");
		given(game.getTeamHome().hasPlayer(attacker)).willReturn(true);
		given(game.getPlayerById("p2")).willReturn(defender);
		given(defender.getName()).willReturn("Jane");
		given(game.getTeamHome().hasPlayer(defender)).willReturn(false);

		ReportSelectBlitzTarget report = new ReportSelectBlitzTarget("p1", "p2");
		List<Run> runs = render(new SelectBlitzTargetMessage(), report);

		Run janeRun = runs.stream().filter(r -> "Jane".equals(r.text)).findFirst().orElseThrow();
		assertEquals(TextStyle.AWAY, janeRun.textStyle);
	}
}
