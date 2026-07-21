package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.mixed.ReportThrownKeg;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class ThrownKegMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player thrower;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player target;

	@Test
	public void successHitsTarget() {
		given(game.getPlayerById("thrower")).willReturn(thrower);
		given(thrower.getName()).willReturn("Thrower");
		given(game.getTeamHome().hasPlayer(thrower)).willReturn(true);
		given(game.getPlayerById("target")).willReturn(target);
		given(target.getName()).willReturn("Target");
		given(game.getTeamHome().hasPlayer(target)).willReturn(false);

		ReportThrownKeg report = new ReportThrownKeg("thrower", "target", 5, true, false);
		List<Run> runs = render(new ThrownKegMessage(), report);

		assertEquals("Beer Barrel Bash Roll [ 5 ]", runs.get(0).text);
		assertEquals("Thrower", runs.get(2).text);
		assertEquals(" hits ", runs.get(3).text);
		assertEquals("Target", runs.get(4).text);
		assertEquals(".", runs.get(5).text);
	}

	@Test
	public void fumbleHitsSelf() {
		given(game.getPlayerById("thrower")).willReturn(thrower);
		given(thrower.getName()).willReturn("Thrower");
		given(thrower.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(thrower)).willReturn(true);

		ReportThrownKeg report = new ReportThrownKeg("thrower", null, 1, false, true);
		List<Run> runs = render(new ThrownKegMessage(), report);

		assertEquals("himself", runs.get(4).text);
	}

	@Test
	public void missHitsNoOne() {
		given(game.getPlayerById("thrower")).willReturn(thrower);
		given(thrower.getName()).willReturn("Thrower");
		given(game.getTeamHome().hasPlayer(thrower)).willReturn(true);

		ReportThrownKeg report = new ReportThrownKeg("thrower", null, 2, false, false);
		List<Run> runs = render(new ThrownKegMessage(), report);

		assertEquals("no one", runs.get(4).text);
	}
}
