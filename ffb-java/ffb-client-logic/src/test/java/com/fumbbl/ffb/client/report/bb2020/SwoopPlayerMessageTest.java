package com.fumbbl.ffb.client.report.bb2020;

import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.bb2020.ReportSwoopPlayer;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class SwoopPlayerMessageTest extends ReportMessageTestBase {

	@Test
	public void rendersRollAndSquares() {
		given(client.getUserInterface().getPitchDimensionProvider().mapToLocal(Direction.EAST)).willReturn(Direction.EAST);

		ReportSwoopPlayer report = new ReportSwoopPlayer(new FieldCoordinate(5, 7), new FieldCoordinate(8, 7), Direction.EAST, 3);
		List<Run> runs = render(new SwoopPlayerMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());

		assertTrue(texts.contains("Swoop Roll [ 3 ] in direction East"));
		assertTrue(texts.contains("Player swoops from square (5,7) to square (8,7)."));
	}

	@Test
	public void differentDirectionAndDistance() {
		given(client.getUserInterface().getPitchDimensionProvider().mapToLocal(Direction.NORTH)).willReturn(Direction.NORTH);

		ReportSwoopPlayer report = new ReportSwoopPlayer(new FieldCoordinate(0, 0), new FieldCoordinate(0, 2), Direction.NORTH, 2);
		List<Run> runs = render(new SwoopPlayerMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());

		assertTrue(texts.contains("Swoop Roll [ 2 ] in direction North"));
		assertTrue(texts.contains("Player swoops from square (0,0) to square (0,2)."));
	}

	@Test
	public void rollLineUsesRollStyle() {
		given(client.getUserInterface().getPitchDimensionProvider().mapToLocal(Direction.SOUTHEAST)).willReturn(Direction.SOUTHEAST);

		ReportSwoopPlayer report = new ReportSwoopPlayer(new FieldCoordinate(1, 1), new FieldCoordinate(2, 2), Direction.SOUTHEAST, 1);
		List<Run> runs = render(new SwoopPlayerMessage(), report);

		Run rollRun = runs.stream().filter(r -> r.text != null).findFirst().orElseThrow();
		assertEquals(TextStyle.ROLL, rollRun.textStyle);
	}
}
