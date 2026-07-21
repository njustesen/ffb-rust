package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.report.ReportPilingOn;

import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class PilingOnMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Mock
	private Skill pilingOnSkill;

	@Test
	public void playerWithoutSkillRendersNothing() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getSkillWithProperty(NamedProperties.canPileOnOpponent)).willReturn(null);

		ReportPilingOn report = new ReportPilingOn("p1", true, false);
		List<Run> runs = render(new PilingOnMessage(), report);

		assertTrue(runs.isEmpty());
	}

	@SuppressWarnings("unchecked")
	@Test
	public void usedReRollInjuryReportsInjury() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getSkillWithProperty(NamedProperties.canPileOnOpponent)).willReturn(pilingOnSkill);
		given(pilingOnSkill.getName()).willReturn("PilingOn");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		given(player.getName()).willReturn("Player p1");

		ReportPilingOn report = new ReportPilingOn("p1", true, true);
		List<Run> runs = render(new PilingOnMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " uses PilingOn to re-roll Injury.".equals(r.text)));
	}

	@SuppressWarnings("unchecked")
	@Test
	public void usedReRollArmorReportsArmor() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getSkillWithProperty(NamedProperties.canPileOnOpponent)).willReturn(pilingOnSkill);
		given(pilingOnSkill.getName()).willReturn("PilingOn");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		given(player.getName()).willReturn("Player p1");

		ReportPilingOn report = new ReportPilingOn("p1", true, false);
		List<Run> runs = render(new PilingOnMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " uses PilingOn to re-roll Armor.".equals(r.text)));
	}

	@SuppressWarnings("unchecked")
	@Test
	public void notUsedReportsDoesNotUse() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getSkillWithProperty(NamedProperties.canPileOnOpponent)).willReturn(pilingOnSkill);
		given(pilingOnSkill.getName()).willReturn("PilingOn");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		given(player.getName()).willReturn("Player p1");

		ReportPilingOn report = new ReportPilingOn("p1", false, false);
		List<Run> runs = render(new PilingOnMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " does not use PilingOn.".equals(r.text)));
	}

	@Test
	public void unknownPlayerIdRendersNothing() {
		given(game.getPlayerById("unknown")).willReturn(null);

		ReportPilingOn report = new ReportPilingOn("unknown", true, false);
		List<Run> runs = render(new PilingOnMessage(), report);

		assertTrue(runs.isEmpty());
	}
}
