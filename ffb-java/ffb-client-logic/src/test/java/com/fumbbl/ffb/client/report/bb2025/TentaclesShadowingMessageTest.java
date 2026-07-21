package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.report.mixed.ReportTentaclesShadowingRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class TentaclesShadowingMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player attacker;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player defender;

	@Mock
	private Skill skill;

	private void stubShadowing() {
		given(skill.hasSkillProperty(NamedProperties.canFollowPlayerLeavingTacklezones)).willReturn(true);
		given(skill.hasSkillProperty(NamedProperties.canHoldPlayersLeavingTacklezones)).willReturn(false);
	}

	private void stubTentacles() {
		given(skill.hasSkillProperty(NamedProperties.canFollowPlayerLeavingTacklezones)).willReturn(false);
		given(skill.hasSkillProperty(NamedProperties.canHoldPlayersLeavingTacklezones)).willReturn(true);
	}

	private void stubCommon() {
		given(game.getPlayerById("defender")).willReturn(defender);
		given(game.getActingPlayer().getPlayer()).willReturn(attacker);
		given(defender.getPlayerGender()).willReturn(PlayerGender.MALE);
	}

	@Test
	public void reportIdIsTentaclesShadowingRoll() {
		assertEquals(ReportId.TENTACLES_SHADOWING_ROLL.getKey(), new TentaclesShadowingMessage().getKey());
	}

	@Test
	public void shadowingSuccessReportsRollAndNeededRoll() {
		stubCommon();
		stubShadowing();

		ReportTentaclesShadowingRoll report = new ReportTentaclesShadowingRoll(skill, "defender", 4, true, 3, false);
		List<Run> runs = render(new TentaclesShadowingMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).filter(Objects::nonNull).collect(Collectors.toList());

		assertTrue(texts.stream().anyMatch(t -> t.equals("Shadowing Roll [ 4 ]")));
		assertTrue(texts.stream().anyMatch(t -> t.contains("shadows his opponent successfully.")));
		assertTrue(texts.stream().anyMatch(t -> t.equals("Succeeded on a roll of 3+")));
	}

	@Test
	public void shadowingFailureReportsRollAMessage() {
		stubCommon();
		stubShadowing();

		ReportTentaclesShadowingRoll report = new ReportTentaclesShadowingRoll(skill, "defender", 1, false, 3, false);
		List<Run> runs = render(new TentaclesShadowingMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).filter(Objects::nonNull).collect(Collectors.toList());

		assertTrue(texts.stream().anyMatch(t -> t.contains("fails to shadow his opponent.")));
		assertTrue(texts.stream().anyMatch(t -> t.equals("Roll a 3+ to succeed")));
	}

	@Test
	public void tentaclesSuccessReportsStComparison() {
		stubCommon();
		stubTentacles();
		given(defender.getStrengthWithModifiers()).willReturn(3);
		given(attacker.getStrengthWithModifiers()).willReturn(3);

		ReportTentaclesShadowingRoll report = new ReportTentaclesShadowingRoll(skill, "defender", 6, true, 4, false);
		List<Run> runs = render(new TentaclesShadowingMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).filter(Objects::nonNull).collect(Collectors.toList());

		assertTrue(texts.stream().anyMatch(t -> t.equals("Tentacles Roll [ 6 ]")));
		assertTrue(texts.stream().anyMatch(t -> t.contains("holds his opponent successfully.")));
		assertTrue(texts.stream().anyMatch(t -> t.contains("(Roll + ST 3 - ST 3 >= 6).")));
	}

	@Test
	public void reRolledSkipsIntroLinesAndNeededRoll() {
		stubCommon();
		stubShadowing();

		ReportTentaclesShadowingRoll report = new ReportTentaclesShadowingRoll(skill, "defender", 4, true, 3, true);
		List<Run> runs = render(new TentaclesShadowingMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).filter(Objects::nonNull).collect(Collectors.toList());

		assertFalse(texts.stream().anyMatch(t -> t.contains("tries to shadow")));
		assertFalse(texts.stream().anyMatch(t -> t.contains("Succeeded on a roll of")));
	}
}
