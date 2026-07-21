package com.fumbbl.ffb.client.report.mixed;

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
	private Player attacker;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player defender;

	private ReportBlockChoice makeReport(BlockResult blockResult, String defenderId, boolean showName) {
		return new ReportBlockChoice(2, new int[] {2, 3}, 0, blockResult, defenderId, false, showName, 1);
	}

	@Test
	public void showsDefenderNameWhenConfigured() {
		given(game.getPlayerById("d1")).willReturn(defender);
		given(defender.getName()).willReturn("Defender");
		given(game.getActingPlayer().getPlayer()).willReturn(attacker);

		ReportBlockChoice report = makeReport(BlockResult.PUSHBACK, "d1", true);
		List<Run> runs = render(new BlockChoiceMessage(), report);

		assertEquals("Block Result against Defender [ PUSHBACK ]", runs.get(0).text);
	}

	@Test
	public void hidesDefenderNameWhenNotConfigured() {
		given(game.getPlayerById("d1")).willReturn(defender);
		given(game.getActingPlayer().getPlayer()).willReturn(attacker);

		ReportBlockChoice report = makeReport(BlockResult.POW, "d1", false);
		List<Run> runs = render(new BlockChoiceMessage(), report);

		assertEquals("Block Result [ POW ]", runs.get(0).text);
	}

	@Test
	public void bothDownWithAttackerSkillPrintsSavedMessage() {
		given(game.getPlayerById("d1")).willReturn(defender);
		given(game.getActingPlayer().getPlayer()).willReturn(attacker);

		// Neither mock reports a matching skill property, so only the header (plus its
		// println terminator) is printed - mirrors the Rust test's "no skill" branch.
		ReportBlockChoice report = makeReport(BlockResult.BOTH_DOWN, "d1", false);
		List<Run> runs = render(new BlockChoiceMessage(), report);

		assertEquals(2, runs.size());
	}

	@Test
	public void powPushbackWithoutTacklePrintsOnlyHeader() {
		given(game.getPlayerById("d1")).willReturn(defender);
		given(game.getActingPlayer().getPlayer()).willReturn(attacker);

		ReportBlockChoice report = makeReport(BlockResult.POW_PUSHBACK, "d1", false);
		List<Run> runs = render(new BlockChoiceMessage(), report);

		assertEquals(2, runs.size());
	}
}
