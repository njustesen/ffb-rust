package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.ReRollSources;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.factory.SkillFactory;
import com.fumbbl.ffb.model.GameRules;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportReRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class ReRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Mock
	private GameRules rules;

	@Mock
	private SkillFactory skillFactory;

	private void givenPlayer(PlayerGender gender) {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Grombrindal");
		given(player.getPlayerGender()).willReturn(gender);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
	}

	@Test
	public void renderLonerSuccessful() {
		givenPlayer(PlayerGender.MALE);

		ReportReRoll report = new ReportReRoll("p1", ReRollSources.LONER, true, 4);
		List<Run> runs = render(new ReRollMessage(), report);

		assertEquals("Loner Roll [ 4 ]", runs.get(0).text);
		assertEquals(TextStyle.ROLL, runs.get(0).textStyle);
		assertEquals(" may use a Team Re-Roll.", runs.get(3).text);
	}

	@Test
	public void renderLonerFailed() {
		givenPlayer(PlayerGender.MALE);

		ReportReRoll report = new ReportReRoll("p1", ReRollSources.LONER, false, 1);
		List<Run> runs = render(new ReRollMessage(), report);

		assertEquals(" wastes a Team Re-Roll.", runs.get(3).text);
	}

	@Test
	public void renderProSuccessfulUsesDative() {
		givenPlayer(PlayerGender.FEMALE);

		ReportReRoll report = new ReportReRoll("p1", ReRollSources.PRO, true, 4);
		List<Run> runs = render(new ReRollMessage(), report);

		assertEquals("Pro Roll [ 4 ]", runs.get(0).text);
		assertEquals("'s Pro skill allows her to re-roll the action.", runs.get(3).text);
	}

	@Test
	public void renderProFailedUsesDative() {
		givenPlayer(PlayerGender.NONBINARY);

		ReportReRoll report = new ReportReRoll("p1", ReRollSources.PRO, false, 1);
		List<Run> runs = render(new ReRollMessage(), report);

		assertEquals("'s Pro skill does not help them.", runs.get(3).text);
	}

	@Test
	public void renderOtherSourcePrintsUppercasedName() {
		given(game.getRules()).willReturn(rules);
		given(rules.getSkillFactory()).willReturn(skillFactory);
		given(skillFactory.forName("Team ReRoll")).willReturn(null);

		ReportReRoll report = new ReportReRoll(null, ReRollSources.TEAM_RE_ROLL, true, 3);
		List<Run> runs = render(new ReRollMessage(), report);

		assertEquals("Re-Roll using TEAM REROLL", runs.get(0).text);
	}
}
