package com.fumbbl.ffb.client.report.bb2016;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.PassingDistance;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.factory.PassModifierFactory;
import com.fumbbl.ffb.mechanics.Mechanic;
import com.fumbbl.ffb.mechanics.PassResult;
import com.fumbbl.ffb.mechanics.bb2016.PassMechanic;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.bb2016.ReportPassRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;
import org.mockito.Mockito;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class PassRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player thrower;

	private void stubMechanic() {
		given(game.getRules().getFactory(FactoryType.Factory.MECHANIC).forName(Mechanic.Type.PASS.name()))
			.willReturn(new PassMechanic());
		PassModifierFactory passModifierFactory = Mockito.mock(PassModifierFactory.class);
		Mockito.doReturn(passModifierFactory).when(game).getFactory(FactoryType.Factory.PASS_MODIFIER);
	}

	@Test
	public void getKeyIsPassRoll() {
		assertEquals("passRoll", new PassRollMessage().getKey());
	}

	@Test
	public void accuratePassReportsSuccess() {
		given(game.getPlayerById("thrower")).willReturn(thrower);
		given(thrower.getName()).willReturn("Thrower");
		given(game.getTeamHome().hasPlayer(thrower)).willReturn(true);
		stubMechanic();

		ReportPassRoll report = new ReportPassRoll("thrower", 5, 2, false, null, PassingDistance.QUICK_PASS, false, PassResult.ACCURATE);
		List<Run> runs = render(new PassRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " passes the ball.".equals(r.text)));
	}

	@Test
	public void fumbleReportsFumble() {
		given(game.getPlayerById("thrower")).willReturn(thrower);
		given(thrower.getName()).willReturn("Thrower");
		given(game.getTeamHome().hasPlayer(thrower)).willReturn(true);
		stubMechanic();

		ReportPassRoll report = new ReportPassRoll("thrower", 1, 2, false, null, PassingDistance.QUICK_PASS, false, PassResult.FUMBLE);
		List<Run> runs = render(new PassRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " fumbles the ball.".equals(r.text)));
	}

	@Test
	public void hailMaryPassSkipsCatcherLookup() {
		given(game.getPlayerById("thrower")).willReturn(thrower);
		given(thrower.getName()).willReturn("Thrower");
		given(game.getTeamHome().hasPlayer(thrower)).willReturn(true);
		stubMechanic();

		ReportPassRoll report = new ReportPassRoll("thrower", 6, 2, false, null, null, false, PassResult.ACCURATE, true);
		List<Run> runs = render(new PassRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " throws a Hail Mary pass:".equals(r.text)));
	}
}
