package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.mixed.ReportOldPro;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class OldProMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Test
	public void selfInflictedTrue() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Grobnik");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportOldPro report = new ReportOldPro("p1", 2, 5, true);
		List<Run> runs = render(new OldProMessage(), report);

		assertEquals("Old Pro Roll [ 5 ]", runs.get(0).text);
		assertEquals("Grobnik", runs.get(2).text);
		assertEquals(" forced the opponent to re-roll a 2 into a 5.", runs.get(3).text);
	}

	@Test
	public void selfInflictedFalse() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Grobnik");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportOldPro report = new ReportOldPro("p1", 3, 6, false);
		List<Run> runs = render(new OldProMessage(), report);

		assertEquals(" re-rolled a 3 into a 6.", runs.get(3).text);
	}

	@Test
	public void unknownPlayerSkipsNameRun() {
		given(game.getPlayerById("unknown")).willReturn(null);

		ReportOldPro report = new ReportOldPro("unknown", 1, 2, false);
		List<Run> runs = render(new OldProMessage(), report);

		// Only the roll header (println = 2 runs) and the final sentence (println = 2 runs)
		// are emitted; no player-name run since print(indent, bold, null) emits nothing.
		assertEquals(4, runs.size());
	}
}
