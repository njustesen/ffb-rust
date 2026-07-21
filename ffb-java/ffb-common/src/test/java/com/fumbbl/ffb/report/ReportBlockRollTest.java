package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportBlockRollTest {

	private ReportBlockRoll make() {
		return new ReportBlockRoll("team1", new int[]{2, 4, 6}, "def1");
	}

	@Test
	public void serializationRoundTrip() {
		ReportBlockRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportBlockRoll restored = new ReportBlockRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getChoosingTeamId(), restored.getChoosingTeamId());
		assertArrayEquals(original.getBlockRoll(), restored.getBlockRoll());
		assertEquals(original.getDefenderId(), restored.getDefenderId());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("blockRoll", json.get("reportId").asString());
	}
}
