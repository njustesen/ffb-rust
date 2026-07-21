package com.fumbbl.ffb.client.report.bb2016;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.PassingDistance;
import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.factory.PassModifierFactory;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.modifiers.PassModifier;
import com.fumbbl.ffb.report.bb2016.ReportThrowTeamMateRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;
import org.mockito.Mockito;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class ThrowTeamMateRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player thrower;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player thrownPlayer;

	private void stubPassModifierFactory() {
		PassModifierFactory passModifierFactory = Mockito.mock(PassModifierFactory.class);
		Mockito.doReturn(passModifierFactory).when(game).getFactory(FactoryType.Factory.PASS_MODIFIER);
	}

	@Test
	public void getKeyIsThrowTeamMateRoll() {
		assertEquals("throwTeamMateRoll", new ThrowTeamMateRollMessage().getKey());
	}

	@Test
	public void successfulThrowReportsSuccessAndNeededRoll() {
		stubPassModifierFactory();
		given(game.getActingPlayer().getPlayer()).willReturn(thrower);
		given(thrower.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getPlayerById("thrown")).willReturn(thrownPlayer);

		ReportThrowTeamMateRoll report = new ReportThrowTeamMateRoll("thrower", true, 4, 2, false, new PassModifier[0],
			PassingDistance.QUICK_PASS, "thrown");
		List<Run> runs = render(new ThrowTeamMateRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " throws his team-mate successfully.".equals(r.text)));
		Run needed = runs.stream().filter(r -> r.textStyle == TextStyle.NEEDED_ROLL).findFirst().orElseThrow();
		assertEquals("Succeeded on a roll of 2+ to avoid a fumble (Roll  + 1 Quick Pass > 1).", needed.text);
	}

	@Test
	public void failedThrowReportsFumble() {
		stubPassModifierFactory();
		given(game.getActingPlayer().getPlayer()).willReturn(thrower);
		given(thrower.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getPlayerById("thrown")).willReturn(thrownPlayer);

		ReportThrowTeamMateRoll report = new ReportThrowTeamMateRoll("thrower", false, 1, 2, false, new PassModifier[0],
			PassingDistance.QUICK_PASS, "thrown");
		List<Run> runs = render(new ThrowTeamMateRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " fumbles the throw.".equals(r.text)));
	}

	@Test
	public void indentIncrementedAfterRender() {
		stubPassModifierFactory();
		given(game.getActingPlayer().getPlayer()).willReturn(thrower);
		given(thrower.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getPlayerById("thrown")).willReturn(thrownPlayer);

		ReportThrowTeamMateRoll report = new ReportThrowTeamMateRoll("thrower", true, 4, 2, true, new PassModifier[0],
			null, "thrown");
		int before = statusReport.getIndent();
		render(new ThrowTeamMateRollMessage(), report);

		assertEquals(before + 1, statusReport.getIndent());
	}
}
