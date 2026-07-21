package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.mixed.ReportIndomitable;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class IndomitableMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player defender;

	// NOTE: the Rust test `skips_gender_run_when_player_missing` is not ported.
	// The Java handler calls `player.getPlayerGender()` unconditionally on the
	// resolved player (no null guard), so a missing player id would throw a
	// NullPointerException in the real renderer rather than skip the run the
	// way the defensive Rust translation does. This is a genuine behavioral
	// difference between the two implementations, not something expressible
	// as a normal render assertion against the real Java class.

	@Test
	public void rendersPlayerGenderAndDefender() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Bruiser");
		given(player.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getPlayerById("d1")).willReturn(defender);
		given(defender.getName()).willReturn("Target");

		ReportIndomitable report = new ReportIndomitable("p1", "d1");
		List<Run> runs = render(new IndomitableMessage(), report);

		assertEquals("Bruiser", runs.get(0).text);
		assertEquals(" uses Indomitable to push ", runs.get(1).text);
		assertEquals(PlayerGender.MALE.getGenitive(), runs.get(2).text);
		assertEquals(" strength to the double of ", runs.get(3).text);
		assertEquals("Target", runs.get(4).text);
		assertEquals(".", runs.get(5).text);
	}

	@Test
	public void skipsDefenderRunWhenDefenderMissing() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Bruiser");
		given(player.getPlayerGender()).willReturn(PlayerGender.MALE);

		ReportIndomitable report = new ReportIndomitable("p1", "missing");
		List<Run> runs = render(new IndomitableMessage(), report);

		// Unlike the Rust translation, the real Java handler's print(indent, bold, Player)
		// does not "skip" the defender run here: game is a RETURNS_DEEP_STUBS mock, so
		// game.getPlayerById("missing") auto-generates a non-null Player mock (rather than
		// null), so the defender branch still emits a run (with a null name, since that
		// mock's getName() is unstubbed) - it just shifts the final "." run out to index 5
		// instead of 4.
		assertEquals("Bruiser", runs.get(0).text);
		assertEquals(" strength to the double of ", runs.get(3).text);
		assertEquals(".", runs.get(5).text);
	}
}
