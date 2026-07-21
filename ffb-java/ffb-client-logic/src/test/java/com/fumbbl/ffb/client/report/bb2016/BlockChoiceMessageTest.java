package com.fumbbl.ffb.client.report.bb2016;

import com.fumbbl.ffb.BlockResult;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportBlockChoice;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class BlockChoiceMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player defender;

	@Test
	public void getKeyIsBlockChoice() {
		assertEquals("blockChoice", new BlockChoiceMessage().getKey());
	}

	@Test
	public void reportsBlockResultWithoutName() {
		given(game.getPlayerById("defender")).willReturn(defender);

		ReportBlockChoice report = new ReportBlockChoice(1, new int[]{3}, 0, BlockResult.PUSHBACK, "defender", false, false, 1);
		List<Run> runs = render(new BlockChoiceMessage(), report);

		assertEquals("Block Result [ PUSHBACK ]", runs.get(0).text);
	}

	@Test
	public void reportsBlockResultWithDefenderName() {
		given(game.getPlayerById("defender")).willReturn(defender);
		given(defender.getName()).willReturn("Player defender");

		ReportBlockChoice report = new ReportBlockChoice(1, new int[]{3}, 0, BlockResult.PUSHBACK, "defender", false, true, 1);
		List<Run> runs = render(new BlockChoiceMessage(), report);

		assertEquals("Block Result against Player defender [ PUSHBACK ]", runs.get(0).text);
	}
}
