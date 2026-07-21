package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportShowStarReRollTest {

	private ReportShowStarReRoll make() {
		return new ReportShowStarReRoll("p1");
	}

	@Test
	public void serializationRoundTrip() {
		ReportShowStarReRoll original = make();
		JsonObject json = original.toJsonValue().asObject();
		ReportShowStarReRoll restored = new ReportShowStarReRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue().asObject();
		assertEquals("showStarReRoll", json.get("reportId").asString());
	}
}
