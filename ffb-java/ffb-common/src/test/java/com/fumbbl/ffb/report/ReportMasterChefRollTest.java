package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportMasterChefRollTest {

	private ReportMasterChefRoll make() {
		return new ReportMasterChefRoll("team1", new int[]{4, 5, 3}, 2);
	}

	@Test
	public void serializationRoundTrip() {
		ReportMasterChefRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportMasterChefRoll restored = new ReportMasterChefRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getTeamId(), restored.getTeamId());
		assertArrayEquals(original.getMasterChefRoll(), restored.getMasterChefRoll());
		assertEquals(original.getReRollsStolen(), restored.getReRollsStolen());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("masterChefRoll", json.get("reportId").asString());
	}
}
