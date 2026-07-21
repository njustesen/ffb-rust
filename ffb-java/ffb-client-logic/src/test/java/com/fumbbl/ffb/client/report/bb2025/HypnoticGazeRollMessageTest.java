package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.ActingPlayer;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.modifiers.RollModifier;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.report.mixed.ReportHypnoticGazeRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class HypnoticGazeRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player hypnotist;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player victim;

	@Mock
	private ActingPlayer actingPlayer;

	private void setUpActingPlayer() {
		given(game.getActingPlayer()).willReturn(actingPlayer);
		given(actingPlayer.getPlayer()).willReturn(hypnotist);
		given(game.getTeamHome().hasPlayer(hypnotist)).willReturn(true);
		given(game.getTeamHome().hasPlayer(victim)).willReturn(false);
		given(hypnotist.getPlayerGender()).willReturn(PlayerGender.MALE);
	}

	@Test
	public void successfulGazeNotRerolledShowsNeededRollAndGazeLines() {
		setUpActingPlayer();
		given(game.getDefender()).willReturn(victim);

		ReportHypnoticGazeRoll report = new ReportHypnoticGazeRoll("a1", true, 6, 2, false, new RollModifier[0], null);
		List<Run> runs = render(new HypnoticGazeRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertTrue(texts.contains(" gazes upon "));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("hypnotizes")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("Succeeded on a roll of 2+")));
	}

	@Test
	public void failedGazeNotRerolledShowsRollToSucceed() {
		setUpActingPlayer();
		given(game.getDefender()).willReturn(victim);

		ReportHypnoticGazeRoll report = new ReportHypnoticGazeRoll("a1", false, 1, 2, false, new RollModifier[0], null);
		List<Run> runs = render(new HypnoticGazeRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("fails to affect")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("Roll a 2+ to succeed")));
	}

	@Test
	public void reRolledSkipsGazeIntroAndNeededRoll() {
		setUpActingPlayer();

		ReportHypnoticGazeRoll report = new ReportHypnoticGazeRoll("a1", true, 6, 2, true, new RollModifier[0], null);
		List<Run> runs = render(new HypnoticGazeRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertFalse(texts.contains(" gazes upon "));
		assertFalse(texts.stream().anyMatch(t -> t != null && t.contains("Succeeded on a roll")));
	}

	@Test
	public void explicitDefenderIdUsedOverGameDefender() {
		setUpActingPlayer();
		given(game.getPlayerById("d1")).willReturn(victim);
		given(victim.getName()).willReturn("Victim");

		ReportHypnoticGazeRoll report = new ReportHypnoticGazeRoll("a1", true, 6, 2, false, new RollModifier[0], "d1");
		List<Run> runs = render(new HypnoticGazeRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());
		assertTrue(texts.stream().anyMatch(t -> "Victim".equals(t)));
	}

	@Test
	public void reportIdIsHypnoticGazeRoll() {
		assertEquals(ReportId.HYPNOTIC_GAZE_ROLL.getKey(), new HypnoticGazeRollMessage().getKey());
	}
}
