package com.fumbbl.ffb.client.report.bb2016;

import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.bb2016.ReportSwoopPlayer;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class SwoopPlayerMessageTest extends ReportMessageTestBase {

	@Test
	public void getKeyIsSwoopPlayer() {
		assertEquals("swoopPlayer", new SwoopPlayerMessage().getKey());
	}

	@Test
	public void reportsSwoopAndMovement() {
		given(client.getUserInterface().getPitchDimensionProvider().mapToLocal(Direction.WEST)).willReturn(Direction.WEST);

		ReportSwoopPlayer report = new ReportSwoopPlayer(new FieldCoordinate(1, 2), new FieldCoordinate(3, 4),
			new Direction[]{Direction.WEST}, new int[]{4});
		List<Run> runs = render(new SwoopPlayerMessage(), report);

		assertEquals("Swoop Roll [ 4 ] West", runs.get(0).text);
		assertEquals("Player swoops from square (1,2) to square (3,4).", runs.get(2).text);
	}

	@Test
	public void emptyRollsProduceNoOutput() {
		ReportSwoopPlayer report = new ReportSwoopPlayer(new FieldCoordinate(0, 0), new FieldCoordinate(0, 0),
			new Direction[0], new int[0]);
		List<Run> runs = render(new SwoopPlayerMessage(), report);

		assertTrue(runs.isEmpty());
	}

	@Test
	public void multipleRollsUsePluralLabel() {
		given(client.getUserInterface().getPitchDimensionProvider().mapToLocal(Direction.EAST)).willReturn(Direction.EAST);
		given(client.getUserInterface().getPitchDimensionProvider().mapToLocal(Direction.SOUTH)).willReturn(Direction.SOUTH);

		ReportSwoopPlayer report = new ReportSwoopPlayer(new FieldCoordinate(1, 1), new FieldCoordinate(2, 2),
			new Direction[]{Direction.EAST, Direction.SOUTH}, new int[]{3, 5});
		List<Run> runs = render(new SwoopPlayerMessage(), report);

		assertEquals("Swoop Rolls [ 3, 5 ] East, South", runs.get(0).text);
	}
}
