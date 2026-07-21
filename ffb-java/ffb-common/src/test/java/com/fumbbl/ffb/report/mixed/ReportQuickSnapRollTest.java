package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportQuickSnapRollTest {

	private ReportQuickSnapRoll make() {
		return new ReportQuickSnapRoll("team1", 2, 3);
	}

	@Test
	public void serializationRoundTrip() {
		ReportQuickSnapRoll original = make();
		JsonObject json = original.toJsonValue().asObject();
		ReportQuickSnapRoll restored =
			(ReportQuickSnapRoll) new ReportQuickSnapRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getTeamId(), restored.getTeamId());
		assertEquals(original.getAmount(), restored.getAmount());
		assertEquals(original.getRoll(), restored.getRoll());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue().asObject();
		assertEquals("quickSnapRoll", json.get("reportId").asString());
	}
}
