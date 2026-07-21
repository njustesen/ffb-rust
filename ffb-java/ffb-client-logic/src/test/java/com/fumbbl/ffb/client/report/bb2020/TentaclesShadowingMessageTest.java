package com.fumbbl.ffb.client.report.bb2020;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.ActingPlayer;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.report.mixed.ReportTentaclesShadowingRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class TentaclesShadowingMessageTest extends ReportMessageTestBase {

	@Mock
	private ActingPlayer actingPlayer;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player actingPlayerModel;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player defender;

	@Mock
	private Skill skill;

	private void baseStubs() {
		given(game.getActingPlayer()).willReturn(actingPlayer);
		given(actingPlayer.getPlayer()).willReturn(actingPlayerModel);
		given(game.getPlayerById("defender")).willReturn(defender);
		given(defender.getPlayerGender()).willReturn(PlayerGender.FEMALE);
		given(defender.getMovementWithModifiers()).willReturn(6);
		given(actingPlayerModel.getMovementWithModifiers()).willReturn(7);
		given(defender.getStrengthWithModifiers()).willReturn(4);
		given(actingPlayerModel.getStrengthWithModifiers()).willReturn(3);
	}

	@Test
	public void shadowingSkillSuccess() {
		baseStubs();
		given(skill.hasSkillProperty(NamedProperties.canFollowPlayerLeavingTacklezones)).willReturn(true);
		given(skill.hasSkillProperty(NamedProperties.canHoldPlayersLeavingTacklezones)).willReturn(false);

		ReportTentaclesShadowingRoll report = new ReportTentaclesShadowingRoll(skill, "defender", 6, true, 4, false);
		List<Run> runs = render(new TentaclesShadowingMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());

		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("tries to shadow")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("Shadowing Roll [ 6 ]")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("shadows her opponent successfully.")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("Succeeded on a roll of 4+")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("MA 6 - MA 7 >= 6")));
	}

	@Test
	public void shadowingSkillFailure() {
		baseStubs();
		given(skill.hasSkillProperty(NamedProperties.canFollowPlayerLeavingTacklezones)).willReturn(true);
		given(skill.hasSkillProperty(NamedProperties.canHoldPlayersLeavingTacklezones)).willReturn(false);

		ReportTentaclesShadowingRoll report = new ReportTentaclesShadowingRoll(skill, "defender", 2, false, 4, false);
		List<Run> runs = render(new TentaclesShadowingMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());

		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("fails to shadow her opponent.")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("Roll a 4+ to succeed")));
	}

	@Test
	public void tentaclesSkillSuccess() {
		baseStubs();
		given(skill.hasSkillProperty(NamedProperties.canFollowPlayerLeavingTacklezones)).willReturn(false);
		given(skill.hasSkillProperty(NamedProperties.canHoldPlayersLeavingTacklezones)).willReturn(true);

		ReportTentaclesShadowingRoll report = new ReportTentaclesShadowingRoll(skill, "defender", 6, true, 4, false);
		List<Run> runs = render(new TentaclesShadowingMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());

		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("tries to hold")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("with her tentacles:")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("Tentacles Roll [ 6 ]")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("holds her opponent successfully.")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("ST 4 - ST 3 >= 6")));
	}

	@Test
	public void reRolledSkipsIntroAndNeededRoll() {
		baseStubs();
		given(skill.hasSkillProperty(NamedProperties.canFollowPlayerLeavingTacklezones)).willReturn(true);
		given(skill.hasSkillProperty(NamedProperties.canHoldPlayersLeavingTacklezones)).willReturn(false);

		ReportTentaclesShadowingRoll report = new ReportTentaclesShadowingRoll(skill, "defender", 6, true, 4, true);
		List<Run> runs = render(new TentaclesShadowingMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());

		assertFalse(texts.stream().anyMatch(t -> t != null && t.contains("tries to shadow")));
		assertFalse(texts.stream().anyMatch(t -> t != null && t.contains("Succeeded on a roll")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("Shadowing Roll [ 6 ]")));
	}
}
