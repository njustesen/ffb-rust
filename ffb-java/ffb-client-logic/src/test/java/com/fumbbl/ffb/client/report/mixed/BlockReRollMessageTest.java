package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.ReRollSources;
import com.fumbbl.ffb.client.ParagraphStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.factory.BlockResultFactory;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.mixed.ReportBlockReRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.BDDMockito.given;

class BlockReRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@org.junit.jupiter.api.BeforeEach
	public void setUpFactory() {
		// game.getRules().getFactory(...) is a generic method (<T extends INamedObjectFactory> T),
		// so the handler's assignment to BlockResultFactory compiles to a checkcast at runtime.
		// A plain given(...).willReturn(...) on this deep-stub chain does not reliably override
		// the auto-generated deep-stub return value, causing a ClassCastException when the
		// handler casts it. Force it with doReturn on the getRules() mock instead.
		given(game.getRules().getFactory(FactoryType.Factory.BLOCK_RESULT)).willReturn(new BlockResultFactory());
		// Force ReRollSource.getName(game) to fall back to its raw name field, matching the
		// Rust test which builds a bare ReRollSource with no skill lookup involved.
		given(game.getRules().getSkillFactory().forName(anyString())).willReturn(null);
	}

	@Test
	public void singleDieUsesSingularWording() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Reroller");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportBlockReRoll report = new ReportBlockReRoll(new int[] {2}, "p1", ReRollSources.TEAM_RE_ROLL);
		List<Run> runs = render(new BlockReRollMessage(), report);

		assertEquals("Re-Rolled Block Dice [ BOTH DOWN ]", runs.get(0).text);
		assertEquals("Reroller", runs.get(2).text);
		assertEquals(" re-rolled 1 block die using Team ReRoll.", runs.get(3).text);
	}

	@Test
	public void multipleDiceUsePluralWordingAndJoinResults() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Reroller");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportBlockReRoll report = new ReportBlockReRoll(new int[] {1, 6}, "p1", ReRollSources.PRO);
		List<Run> runs = render(new BlockReRollMessage(), report);

		assertEquals("Re-Rolled Block Dice [ SKULL ] [ POW ]", runs.get(0).text);
		assertEquals(" re-rolled 2 block dice using Pro.", runs.get(3).text);
	}

	@Test
	public void indentIsHardcodedRegardlessOfStatusReportIndent() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Reroller");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		statusReport.setIndent(5);

		// Java's BlockReRollMessage always dereferences getReRollSource(), so a null source
		// (as used in the Rust test) would NPE here - use a real source instead; only the
		// hardcoded paragraph-style indices are asserted, matching the Rust test's intent.
		ReportBlockReRoll report = new ReportBlockReRoll(new int[] {3}, "p1", ReRollSources.PRO);
		List<Run> runs = render(new BlockReRollMessage(), report);

		assertEquals(ParagraphStyle.INDENT_2, runs.get(0).paragraphStyle);
		assertEquals(ParagraphStyle.INDENT_3, runs.get(2).paragraphStyle);
	}
}
