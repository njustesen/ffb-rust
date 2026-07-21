package com.fumbbl.ffb.client.report.bb2016;

import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.bb2016.ReportApothecaryRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class ApothecaryRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Test
	public void getKeyIsApothecaryRoll() {
		assertEquals("apothecaryRoll", new ApothecaryRollMessage().getKey());
	}

	@Test
	public void noOutputWhenCasualtyRollAbsent() {
		ReportApothecaryRoll report = new ReportApothecaryRoll("p1", new int[]{}, null, null);
		List<Run> runs = render(new ApothecaryRollMessage(), report);
		assertTrue(runs.isEmpty());
	}

	@Test
	public void reportsCasualtyRollAndPlayerState() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Grubb");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportApothecaryRoll report = new ReportApothecaryRoll("p1", new int[]{3, 4},
			new PlayerState(PlayerState.BADLY_HURT), null);
		List<Run> runs = render(new ApothecaryRollMessage(), report);

		assertEquals("Apothecary used.", runs.get(0).text);
		assertEquals("Casualty Roll [ 3 ][ 4 ]", runs.get(2).text);
		assertEquals("Grubb", runs.get(4).text);
		assertEquals(" has been badly hurt.", runs.get(5).text);
	}
}
