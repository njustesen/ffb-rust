package com.fumbbl.ffb.client.report.bb2020;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.mixed.ReportThenIStartedBlastin;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class ThenIStartedBlastinMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player thrower;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player target;

	@Test
	public void fumblePrintsThrowerGenderSelf() {
		given(game.getPlayerById("thrower")).willReturn(thrower);
		given(game.getPlayerById("target")).willReturn(target);
		given(thrower.getPlayerGender()).willReturn(PlayerGender.MALE);

		ReportThenIStartedBlastin report = new ReportThenIStartedBlastin("thrower", "target", 5, false, true);
		List<Run> runs = render(new ThenIStartedBlastinMessage(), report);
		assertTrue(runs.stream().anyMatch(r -> "himself".equals(r.text)));
	}

	@Test
	public void successPrintsTargetPlayer() {
		given(game.getPlayerById("thrower")).willReturn(thrower);
		given(game.getPlayerById("target")).willReturn(target);
		given(target.getName()).willReturn("Player target");

		ReportThenIStartedBlastin report = new ReportThenIStartedBlastin("thrower", "target", 5, true, false);
		List<Run> runs = render(new ThenIStartedBlastinMessage(), report);
		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("Player target")));
	}

	@Test
	public void opponentChosenWhenNotSuccessOrFumble() {
		given(game.getPlayerById("thrower")).willReturn(thrower);
		given(game.getPlayerById("target")).willReturn(target);

		ReportThenIStartedBlastin report = new ReportThenIStartedBlastin("thrower", "target", 5, false, false);
		List<Run> runs = render(new ThenIStartedBlastinMessage(), report);
		assertTrue(runs.stream().anyMatch(r -> "a player chosen by the opposing coach".equals(r.text)));
	}

	@Test
	public void noTargetStartsBlastin() {
		given(game.getPlayerById("thrower")).willReturn(thrower);

		ReportThenIStartedBlastin report = new ReportThenIStartedBlastin("thrower", null, 0, false, false);
		List<Run> runs = render(new ThenIStartedBlastinMessage(), report);
		assertTrue(runs.stream().anyMatch(r -> " starts blastin' ".equals(r.text)));
		// roll <= 0, so no roll-line should be present.
		assertTrue(runs.stream().noneMatch(r -> r.text != null && r.text.contains("Roll [")));
	}
}
