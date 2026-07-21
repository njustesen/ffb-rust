package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportDefectingPlayersTest {

	private ReportDefectingPlayers make() {
		return new ReportDefectingPlayers(new String[]{"p1", "p2"}, new int[]{3, 5}, new boolean[]{true, false});
	}

	@Test
	public void serializationRoundTrip() {
		ReportDefectingPlayers original = make();
		JsonObject json = original.toJsonValue();
		ReportDefectingPlayers restored = new ReportDefectingPlayers().initFrom(ReportTestUtil.source(), json);
		assertArrayEquals(original.getPlayerIds(), restored.getPlayerIds());
		assertArrayEquals(original.getRolls(), restored.getRolls());
		assertArrayEquals(original.getDefectings(), restored.getDefectings());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("defectingPlayers", json.get("reportId").asString());
	}
}
