package com.fumbbl.ffb.client.report.bb2020;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.mechanics.Mechanic;
import com.fumbbl.ffb.mechanics.bb2020.AgilityMechanic;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.modifiers.RollModifier;
import com.fumbbl.ffb.report.mixed.ReportHypnoticGazeRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class HypnoticGazeRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player gazer;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player victim;

	private void stubMechanic() {
		given(game.getRules().getFactory(FactoryType.Factory.MECHANIC).forName(Mechanic.Type.AGILITY.name()))
			.willReturn(new AgilityMechanic());
	}

	@Test
	public void firstRollPrintsGazesUponHeader() {
		given(game.getActingPlayer().getPlayer()).willReturn(gazer);
		given(gazer.getName()).willReturn("Player gazer");
		given(gazer.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(gazer)).willReturn(true);
		given(game.getPlayerById("victim")).willReturn(victim);
		given(victim.getName()).willReturn("Player victim");
		given(game.getTeamHome().hasPlayer(victim)).willReturn(false);
		stubMechanic();

		ReportHypnoticGazeRoll report = new ReportHypnoticGazeRoll("gazer", true, 4, 2, false, new RollModifier<?>[0], "victim");
		List<Run> runs = render(new HypnoticGazeRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());

		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains(" gazes upon ")));
		assertTrue(texts.contains("Player gazer"));
		assertTrue(texts.contains("Player victim"));
	}

	@Test
	public void reRolledSkipsHeaderAndNeededRoll() {
		given(game.getActingPlayer().getPlayer()).willReturn(gazer);
		given(gazer.getName()).willReturn("Player gazer");
		given(gazer.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(gazer)).willReturn(true);
		given(game.getPlayerById("victim")).willReturn(victim);
		given(victim.getName()).willReturn("Player victim");

		ReportHypnoticGazeRoll report = new ReportHypnoticGazeRoll("gazer", true, 4, 2, true, new RollModifier<?>[0], "victim");
		List<Run> runs = render(new HypnoticGazeRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());

		assertTrue(texts.stream().noneMatch(t -> t != null && t.contains(" gazes upon ")));
		assertTrue(texts.stream().noneMatch(t -> t != null && t.contains("Succeeded on a roll of")));
	}

	@Test
	public void successfulRollPrintsHypnotizesWithGenitive() {
		given(game.getActingPlayer().getPlayer()).willReturn(gazer);
		given(gazer.getName()).willReturn("Player gazer");
		given(gazer.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(gazer)).willReturn(true);
		given(game.getPlayerById("victim")).willReturn(victim);
		given(victim.getName()).willReturn("Player victim");
		stubMechanic();

		ReportHypnoticGazeRoll report = new ReportHypnoticGazeRoll("gazer", true, 4, 2, false, new RollModifier<?>[0], "victim");
		List<Run> runs = render(new HypnoticGazeRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());

		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains(" hypnotizes his victim.")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("Succeeded on a roll of 2+")));
	}

	@Test
	public void unsuccessfulRollPrintsFailsToAffect() {
		given(game.getActingPlayer().getPlayer()).willReturn(gazer);
		given(gazer.getName()).willReturn("Player gazer");
		given(gazer.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(gazer)).willReturn(true);
		given(game.getDefender()).willReturn(null);
		stubMechanic();

		ReportHypnoticGazeRoll report = new ReportHypnoticGazeRoll("gazer", false, 1, 3, false, new RollModifier<?>[0], null);
		List<Run> runs = render(new HypnoticGazeRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());

		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains(" fails to affect his victim.")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("Roll a 3+ to succeed")));
	}

	@Test
	public void noDefenderIdFallsBackToGameDefender() {
		given(game.getActingPlayer().getPlayer()).willReturn(gazer);
		given(gazer.getName()).willReturn("Player gazer");
		given(gazer.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(gazer)).willReturn(true);
		given(game.getDefender()).willReturn(victim);
		given(victim.getName()).willReturn("Player victim");
		stubMechanic();

		ReportHypnoticGazeRoll report = new ReportHypnoticGazeRoll("gazer", true, 4, 2, false, new RollModifier<?>[0], null);
		List<Run> runs = render(new HypnoticGazeRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());

		assertTrue(texts.contains("Player victim"));
	}
}
