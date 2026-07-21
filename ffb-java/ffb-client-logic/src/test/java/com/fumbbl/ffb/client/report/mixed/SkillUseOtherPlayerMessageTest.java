package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.SkillUse;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.report.bb2020.ReportSkillUseOtherPlayer;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class SkillUseOtherPlayerMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player user;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player target;

	@Mock
	private Skill skill;

	@Test
	public void rendersSkillUseBetweenPlayers() {
		given(game.getPlayerById("p1")).willReturn(user);
		given(user.getName()).willReturn("User");
		given(user.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(user)).willReturn(true);
		given(game.getPlayerById("p2")).willReturn(target);
		given(target.getName()).willReturn("Target");
		given(game.getTeamHome().hasPlayer(target)).willReturn(false);
		given(skill.getName()).willReturn("Guard");

		ReportSkillUseOtherPlayer report = new ReportSkillUseOtherPlayer("p1", skill, SkillUse.STOP_OPPONENT, "p2");
		List<Run> runs = render(new SkillUseOtherPlayerMessage(), report);

		assertEquals("User", runs.get(0).text);
		assertEquals(" uses Guard of ", runs.get(1).text);
		assertEquals("Target", runs.get(2).text);
		assertEquals(" to stop his opponent.", runs.get(3).text);
	}
}
