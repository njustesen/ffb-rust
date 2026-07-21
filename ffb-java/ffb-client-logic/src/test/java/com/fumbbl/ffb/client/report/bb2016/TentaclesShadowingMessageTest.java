package com.fumbbl.ffb.client.report.bb2016;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.report.bb2016.ReportTentaclesShadowingRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class TentaclesShadowingMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player actingPlayer;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player defender;

	@Mock
	private Skill shadowingSkill;

	@Mock
	private Skill tentaclesSkill;

	@Test
	public void getKeyIsTentaclesShadowingRoll() {
		assertEquals("tentaclesShadowingRoll", new TentaclesShadowingMessage().getKey());
	}

	@Test
	public void shadowingSkillSuccessfulEscape() {
		given(game.getActingPlayer().getPlayer()).willReturn(actingPlayer);
		given(actingPlayer.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getPlayerById("shadower")).willReturn(defender);
		given(shadowingSkill.hasSkillProperty(NamedProperties.canFollowPlayerLeavingTacklezones)).willReturn(true);

		ReportTentaclesShadowingRoll report = new ReportTentaclesShadowingRoll(shadowingSkill, "shadower", new int[]{4, 5}, true, 4, false);
		List<Run> runs = render(new TentaclesShadowingMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " escapes his opponent.".equals(r.text)));
	}

	@Test
	public void tentaclesSkillFailedHold() {
		given(game.getActingPlayer().getPlayer()).willReturn(actingPlayer);
		given(game.getPlayerById("shadower")).willReturn(defender);
		given(defender.getPlayerGender()).willReturn(PlayerGender.FEMALE);
		given(tentaclesSkill.hasSkillProperty(NamedProperties.canHoldPlayersLeavingTacklezones)).willReturn(true);

		ReportTentaclesShadowingRoll report = new ReportTentaclesShadowingRoll(tentaclesSkill, "shadower", new int[]{1, 1}, false, 5, false);
		List<Run> runs = render(new TentaclesShadowingMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " holds her opponent successfully.".equals(r.text)));
	}

	@Test
	public void reRolledSkipsIntroLine() {
		given(game.getActingPlayer().getPlayer()).willReturn(actingPlayer);
		given(actingPlayer.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getPlayerById("shadower")).willReturn(defender);
		given(shadowingSkill.hasSkillProperty(NamedProperties.canFollowPlayerLeavingTacklezones)).willReturn(true);

		ReportTentaclesShadowingRoll report = new ReportTentaclesShadowingRoll(shadowingSkill, "shadower", new int[]{4, 5}, true, 4, true);
		List<Run> runs = render(new TentaclesShadowingMessage(), report);

		assertEquals("Shadowing Escape Roll [ 4 ][ 5 ] = 9", runs.get(0).text);
	}
}
