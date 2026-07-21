package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.report.bb2025.ReportPuntDirection;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class PuntDirectionMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player punter;

	@Test
	public void directionPresentPrintsRollAndDirection() {
		given(client.getUserInterface().getPitchDimensionProvider().mapToLocal(Direction.NORTH)).willReturn(Direction.NORTH);
		given(game.getPlayerById("p1")).willReturn(punter);
		given(punter.getName()).willReturn("Punter");
		given(game.getTeamHome().hasPlayer(punter)).willReturn(true);

		ReportPuntDirection report = new ReportPuntDirection(Direction.NORTH, 3, "p1", false);
		List<Run> runs = render(new PuntDirectionMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.startsWith("Punt Direction Roll [ 3 ]")));
		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("punts the ball")));
	}

	@Test
	public void outOfBoundsAppendsTextWhenDirectionPresent() {
		given(client.getUserInterface().getPitchDimensionProvider().mapToLocal(Direction.NORTH)).willReturn(Direction.NORTH);
		given(game.getPlayerById("p1")).willReturn(punter);
		given(punter.getName()).willReturn("Punter");
		given(game.getTeamHome().hasPlayer(punter)).willReturn(true);

		ReportPuntDirection report = new ReportPuntDirection(Direction.NORTH, 3, "p1", true);
		List<Run> runs = render(new PuntDirectionMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.contains("putting it out of bounds")));
	}

	@Test
	public void noDirectionIntentionallyOutOfBounds() {
		given(game.getPlayerById("p1")).willReturn(punter);
		given(punter.getName()).willReturn("Punter");
		given(game.getTeamHome().hasPlayer(punter)).willReturn(true);

		ReportPuntDirection report = new ReportPuntDirection(null, 0, "p1", true);
		List<Run> runs = render(new PuntDirectionMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " intentionally punts the ball out of bounds.".equals(r.text)));
		assertFalse(runs.stream().anyMatch(r -> r.text != null && r.text.startsWith("Punt Direction Roll")));
	}

	@Test
	public void playerIsPrinted() {
		given(client.getUserInterface().getPitchDimensionProvider().mapToLocal(Direction.NORTH)).willReturn(Direction.NORTH);
		given(game.getPlayerById("p1")).willReturn(punter);
		given(punter.getName()).willReturn("Punter");
		given(game.getTeamHome().hasPlayer(punter)).willReturn(true);

		ReportPuntDirection report = new ReportPuntDirection(Direction.NORTH, 3, "p1", false);
		List<Run> runs = render(new PuntDirectionMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Punter".equals(r.text)));
	}

	@Test
	public void reportIdIsPuntDirectionRoll() {
		assertEquals(ReportId.PUNT_DIRECTION_ROLL.getKey(), new PuntDirectionMessage().getKey());
	}
}
