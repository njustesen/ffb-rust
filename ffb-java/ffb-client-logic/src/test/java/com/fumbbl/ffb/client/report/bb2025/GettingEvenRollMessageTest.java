package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Keyword;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.report.bb2025.ReportGettingEvenRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class GettingEvenRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	private void setUpPlayer() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
	}

	@Test
	public void successfulGainsHatredWithNeededRoll() {
		setUpPlayer();
		ReportGettingEvenRoll report = new ReportGettingEvenRoll("p1", true, 5, 4, false, Keyword.ELF);
		List<Run> runs = render(new GettingEvenRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("gains hatred towards players of type 'Elf'.")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("Succeeded on a roll of 4+")));
	}

	@Test
	public void unsuccessfulRemainsPeacefulWithNeededRoll() {
		setUpPlayer();
		ReportGettingEvenRoll report = new ReportGettingEvenRoll("p1", false, 2, 4, false, Keyword.ELF);
		List<Run> runs = render(new GettingEvenRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("remains peaceful towards players of type 'Elf'.")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("Roll a 4+ to succeed")));
	}

	@Test
	public void reRolledSuppressesNeededRoll() {
		setUpPlayer();
		ReportGettingEvenRoll report = new ReportGettingEvenRoll("p1", true, 5, 4, true, Keyword.ELF);
		List<Run> runs = render(new GettingEvenRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertFalse(texts.stream().anyMatch(t -> t != null && t.contains("Succeeded on a roll of")));
	}

	@Test
	public void neededRollUsesNeededRollStyle() {
		setUpPlayer();
		ReportGettingEvenRoll report = new ReportGettingEvenRoll("p1", true, 5, 4, false, Keyword.ELF);
		List<Run> runs = render(new GettingEvenRollMessage(), report);
		Run run = runs.stream().filter(r -> r.text != null && r.text.contains("Succeeded")).findFirst().orElseThrow();
		assertEquals(TextStyle.NEEDED_ROLL, run.textStyle);
	}

	@Test
	public void reportIdIsGettingEvenRoll() {
		assertEquals(ReportId.GETTING_EVEN_ROLL.getKey(), new GettingEvenRollMessage().getKey());
	}
}
