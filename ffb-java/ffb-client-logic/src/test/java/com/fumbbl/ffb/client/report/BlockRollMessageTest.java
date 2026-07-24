package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.factory.BlockResultFactory;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportBlockRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class BlockRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player defender;

	// rust: empty_roll_renders_nothing
	@Test
	public void emptyRollRendersNothing() {
		ReportBlockRoll report = new ReportBlockRoll("home", new int[]{});
		List<Run> runs = render(new BlockRollMessage(), report);

		assertTrue(runs.isEmpty());
	}

	// rust: roll_without_defender
	@Test
	public void rollWithoutDefender() {
		given(game.getRules().getFactory(FactoryType.Factory.BLOCK_RESULT)).willReturn(new BlockResultFactory());
		ReportBlockRoll report = new ReportBlockRoll("home", new int[]{1, 6});
		List<Run> runs = render(new BlockRollMessage(), report);

		assertEquals("Block Roll [ SKULL ] [ POW ]", runs.get(0).text);
		assertEquals(TextStyle.ROLL, runs.get(0).textStyle);
	}

	// rust: roll_with_defender
	@Test
	public void rollWithDefender() {
		given(game.getRules().getFactory(FactoryType.Factory.BLOCK_RESULT)).willReturn(new BlockResultFactory());
		given(game.getPlayerById("def1")).willReturn(defender);
		given(defender.getName()).willReturn("Defender");
		ReportBlockRoll report = new ReportBlockRoll("home", new int[]{2, 5}, "def1");
		List<Run> runs = render(new BlockRollMessage(), report);

		assertEquals("Block Roll against ", runs.get(0).text);
		assertEquals("Defender", runs.get(1).text);
		assertEquals(" [ BOTH DOWN ] [ POW/PUSH ]", runs.get(2).text);
	}
}
