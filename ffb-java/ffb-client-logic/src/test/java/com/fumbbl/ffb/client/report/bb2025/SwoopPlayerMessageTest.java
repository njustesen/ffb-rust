package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.report.bb2025.ReportSwoopPlayer;
import org.junit.jupiter.api.Test;

import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class SwoopPlayerMessageTest extends ReportMessageTestBase {

	@Test
	public void reportIdIsSwoopPlayer() {
		assertEquals(ReportId.SWOOP_PLAYER.getKey(), new SwoopPlayerMessage().getKey());
	}

	@Test
	public void inBoundsReportsEndSquare() {
		given(client.getUserInterface().getPitchDimensionProvider().mapToLocal(Direction.EAST)).willReturn(Direction.EAST);

		ReportSwoopPlayer report = new ReportSwoopPlayer(new FieldCoordinate(3, 5), new FieldCoordinate(6, 5), Direction.EAST, 3, false);
		List<Run> runs = render(new SwoopPlayerMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).filter(Objects::nonNull).collect(Collectors.toList());

		assertTrue(texts.stream().anyMatch(t -> t.equals("Swoop Roll [ 3 ] in direction East")));
		assertTrue(texts.stream().anyMatch(t -> t.equals("Player swoops from square (3,5) to square (6,5).")));
	}

	@Test
	public void outOfBoundsReportsFans() {
		given(client.getUserInterface().getPitchDimensionProvider().mapToLocal(Direction.NORTH)).willReturn(Direction.NORTH);

		ReportSwoopPlayer report = new ReportSwoopPlayer(new FieldCoordinate(0, 0), new FieldCoordinate(0, 3), Direction.NORTH, 3, true);
		List<Run> runs = render(new SwoopPlayerMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).filter(Objects::nonNull).collect(Collectors.toList());

		// java: the render method always appends ").") after either branch, so the
		// out-of-bounds branch (which already closed with ") into the fans") ends up with
		// a doubled closing paren -- faithfully reproduced here rather than "fixed".
		assertTrue(texts.stream().anyMatch(t -> t.equals("Player swoops from square (0,0) into the fans).")));
	}

	@Test
	public void rollHeaderUsesRollStyle() {
		given(client.getUserInterface().getPitchDimensionProvider().mapToLocal(Direction.SOUTH)).willReturn(Direction.SOUTH);

		ReportSwoopPlayer report = new ReportSwoopPlayer(new FieldCoordinate(1, 1), new FieldCoordinate(1, 4), Direction.SOUTH, 3, false);
		List<Run> runs = render(new SwoopPlayerMessage(), report);

		assertEquals(TextStyle.ROLL, runs.get(0).textStyle);
	}
}
