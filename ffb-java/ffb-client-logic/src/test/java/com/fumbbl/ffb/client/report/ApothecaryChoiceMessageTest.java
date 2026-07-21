package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.SeriousInjury;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportApothecaryChoice;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class ApothecaryChoiceMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Mock
	private SeriousInjury seriousInjury;

	@Test
	public void reserveStatePatchesPlayer() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Patched");
		given(player.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		PlayerState state = new PlayerState(PlayerState.RESERVE);
		ReportApothecaryChoice report = new ReportApothecaryChoice("p1", state, null);
		List<Run> runs = render(new ApothecaryChoiceMessage(), report);

		assertEquals("The apothecary patches ", runs.get(0).text);
		assertEquals("Patched", runs.get(1).text);
		assertEquals(" up so he is able to play again.", runs.get(2).text);
	}

	@Test
	public void keepsOldInjuryWhenStateAndInjuryUnchanged() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		given(game.getTeamHome().getCoach()).willReturn("coach_home");

		PlayerState state = new PlayerState(0);
		given(game.getFieldModel().getPlayerState(player)).willReturn(state);
		given(game.getGameResult().getPlayerResult(player).getSeriousInjury()).willReturn(null);

		ReportApothecaryChoice report = new ReportApothecaryChoice("p1", state, null);
		List<Run> runs = render(new ApothecaryChoiceMessage(), report);

		assertEquals("Coach ", runs.get(0).text);
		assertEquals("coach_home", runs.get(1).text);
		assertEquals(TextStyle.HOME, runs.get(1).textStyle);
		assertEquals(" keeps the old injury result.", runs.get(2).text);
	}

	@Test
	public void choosesNewInjuryWhenSeriousInjuryDiffers() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(game.getTeamHome().hasPlayer(player)).willReturn(false);
		given(game.getTeamAway().getCoach()).willReturn("coach_away");

		PlayerState state = new PlayerState(0);
		given(game.getFieldModel().getPlayerState(player)).willReturn(state);
		given(game.getGameResult().getPlayerResult(player).getSeriousInjury()).willReturn(null);

		ReportApothecaryChoice report = new ReportApothecaryChoice("p1", state, seriousInjury);
		List<Run> runs = render(new ApothecaryChoiceMessage(), report);

		assertEquals(TextStyle.AWAY, runs.get(1).textStyle);
		assertEquals(" chooses the new injury result.", runs.get(2).text);
	}
}
