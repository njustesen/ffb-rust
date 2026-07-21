package com.fumbbl.ffb.client.report.bb2020;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.bb2020.ReportOfficiousRefRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class OfficiousRefRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Test
	public void rollOfOneSendsPlayerOff() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Bob");
		ReportOfficiousRefRoll report = new ReportOfficiousRefRoll(1, "p1");
		List<Run> runs = render(new OfficiousRefRollMessage(), report);
		assertTrue(runs.stream().anyMatch(r -> " is sent off.".equals(r.text)));
	}

	@Test
	public void otherRollStunsPlayer() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Bob");
		ReportOfficiousRefRoll report = new ReportOfficiousRefRoll(4, "p1");
		List<Run> runs = render(new OfficiousRefRollMessage(), report);
		assertTrue(runs.stream().anyMatch(r -> " is stunned".equals(r.text)));
	}

	@Test
	public void rollValueIsReported() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Bob");
		ReportOfficiousRefRoll report = new ReportOfficiousRefRoll(6, "p1");
		List<Run> runs = render(new OfficiousRefRollMessage(), report);
		assertTrue(runs.stream().anyMatch(r -> "Officious Ref Effect Roll [ 6 ]".equals(r.text)));
	}
}
