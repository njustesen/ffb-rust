package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportBlock;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class BlockMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player attacker;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player defender;

	@Test
	public void rendersBlockAction() {
		given(game.getActingPlayer().getPlayer()).willReturn(attacker);
		given(attacker.getName()).willReturn("Attacker");
		given(game.getTeamHome().hasPlayer(attacker)).willReturn(true);
		given(game.getPlayerById("def")).willReturn(defender);
		given(defender.getName()).willReturn("Defender");
		given(game.getTeamHome().hasPlayer(defender)).willReturn(false);
		given(game.getActingPlayer().getPlayerAction()).willReturn(PlayerAction.BLOCK);

		ReportBlock report = new ReportBlock("def");
		List<Run> runs = render(new BlockMessage(), report);

		assertEquals(1, statusReport.getIndent());
		assertEquals("Attacker", runs.get(0).text);
		assertEquals(" blocks ", runs.get(1).text);
		assertEquals("Defender", runs.get(2).text);
		assertEquals(":", runs.get(3).text);
	}

	@Test
	public void rendersBlitzAction() {
		given(game.getActingPlayer().getPlayer()).willReturn(attacker);
		given(attacker.getName()).willReturn("Attacker");
		given(game.getTeamHome().hasPlayer(attacker)).willReturn(true);
		given(game.getPlayerById("def")).willReturn(defender);
		given(defender.getName()).willReturn("Defender");
		given(game.getTeamHome().hasPlayer(defender)).willReturn(false);
		given(game.getActingPlayer().getPlayerAction()).willReturn(PlayerAction.BLITZ);

		ReportBlock report = new ReportBlock("def");
		List<Run> runs = render(new BlockMessage(), report);

		assertEquals(" blitzes ", runs.get(1).text);
	}

	@Test
	public void usesBoldHomeAwayStyles() {
		given(game.getActingPlayer().getPlayer()).willReturn(attacker);
		given(attacker.getName()).willReturn("Attacker");
		given(game.getTeamHome().hasPlayer(attacker)).willReturn(true);
		given(game.getPlayerById("def")).willReturn(defender);
		given(defender.getName()).willReturn("Defender");
		given(game.getTeamHome().hasPlayer(defender)).willReturn(false);
		given(game.getActingPlayer().getPlayerAction()).willReturn(PlayerAction.BLOCK);

		ReportBlock report = new ReportBlock("def");
		List<Run> runs = render(new BlockMessage(), report);

		assertEquals(TextStyle.HOME_BOLD, runs.get(0).textStyle);
		assertEquals(TextStyle.AWAY_BOLD, runs.get(2).textStyle);
	}
}
