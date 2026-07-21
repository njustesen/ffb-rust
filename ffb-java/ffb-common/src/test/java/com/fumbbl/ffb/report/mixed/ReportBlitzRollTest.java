package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportBlitzRollTest {

	private ReportBlitzRoll make() {
		return new ReportBlitzRoll("team1", 4, 2);
	}

	@Test
	public void serializationRoundTrip() {
		ReportBlitzRoll original = make();
		JsonObject json = (JsonObject) original.toJsonValue();
		ReportBlitzRoll restored = (ReportBlitzRoll) new ReportBlitzRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getTeamId(), restored.getTeamId());
		assertEquals(original.getAmount(), restored.getAmount());
		assertEquals(original.getRoll(), restored.getRoll());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = (JsonObject) make().toJsonValue();
		assertEquals("blitzRoll", json.get("reportId").asString());
	}
}
