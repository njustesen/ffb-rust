package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.mixed.ReportCloudBurster;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class CloudBursterMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player thrower;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player interceptor;

	@Test
	public void homeThrowingUsesHomeBoldForThrower() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getPlayerById("t1")).willReturn(thrower);
		given(game.getPlayerById("i1")).willReturn(interceptor);
		given(thrower.getName()).willReturn("Thrower");
		given(interceptor.getName()).willReturn("Interceptor");
		given(interceptor.getPlayerGender()).willReturn(PlayerGender.FEMALE);

		ReportCloudBurster report = new ReportCloudBurster("t1", "i1", "home");
		List<Run> runs = render(new CloudBursterMessage(), report);

		assertEquals("Thrower", runs.get(0).text);
		assertEquals(TextStyle.HOME_BOLD, runs.get(0).textStyle);
		assertEquals(" uses CloudBurster", runs.get(1).text);
		assertEquals("Interceptor", runs.get(3).text);
		assertEquals(TextStyle.AWAY_BOLD, runs.get(3).textStyle);
		assertEquals(" has to reroll her successful interception.", runs.get(4).text);
	}

	@Test
	public void awayThrowingUsesAwayBoldForThrower() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getPlayerById("t1")).willReturn(thrower);
		given(game.getPlayerById("i1")).willReturn(interceptor);
		given(thrower.getName()).willReturn("Thrower");
		given(interceptor.getName()).willReturn("Interceptor");
		given(interceptor.getPlayerGender()).willReturn(PlayerGender.FEMALE);

		ReportCloudBurster report = new ReportCloudBurster("t1", "i1", "away");
		List<Run> runs = render(new CloudBursterMessage(), report);

		assertEquals(TextStyle.AWAY_BOLD, runs.get(0).textStyle);
		assertEquals(TextStyle.HOME_BOLD, runs.get(3).textStyle);
	}
}
