package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.report.bb2025.ReportChompRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class ChompRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player chomper;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player chompee;

	private void setUpPlayers() {
		given(game.getPlayerById("chomper1")).willReturn(chomper);
		given(game.getPlayerById("chompee1")).willReturn(chompee);
		given(game.getTeamHome().hasPlayer(chomper)).willReturn(true);
		given(game.getTeamHome().hasPlayer(chompee)).willReturn(false);
	}

	@Test
	public void successfulChomp() {
		setUpPlayers();
		ReportChompRoll report = new ReportChompRoll("chomper1", true, 5, 3, false, "chomper1", "chompee1");
		List<Run> runs = render(new ChompRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> " chomped ".equals(t)));
	}

	@Test
	public void failedChomp() {
		setUpPlayers();
		ReportChompRoll report = new ReportChompRoll("chomper1", false, 2, 3, false, "chomper1", "chompee1");
		List<Run> runs = render(new ChompRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> " failed to chomp ".equals(t)));
	}

	@Test
	public void printsRollAndBothPlayers() {
		setUpPlayers();
		given(chomper.getName()).willReturn("Chomper");
		given(chompee.getName()).willReturn("Chompee");
		ReportChompRoll report = new ReportChompRoll("chomper1", true, 6, 3, false, "chomper1", "chompee1");
		List<Run> runs = render(new ChompRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> "Chomp Roll [ 6 ]".equals(t)));
		assertTrue(texts.stream().anyMatch(t -> "Chomper".equals(t)));
		assertTrue(texts.stream().anyMatch(t -> "Chompee".equals(t)));
	}

	@Test
	public void reportIdIsChompRoll() {
		assertEquals(ReportId.CHOMP_ROLL.getKey(), new ChompRollMessage().getKey());
	}
}
