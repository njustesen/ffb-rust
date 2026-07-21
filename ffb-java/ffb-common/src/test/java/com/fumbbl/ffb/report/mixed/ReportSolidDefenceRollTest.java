package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportSolidDefenceRollTest {

	private ReportSolidDefenceRoll make() {
		return new ReportSolidDefenceRoll("team1", 3, 2);
	}

	@Test
	public void serializationRoundTrip() {
		ReportSolidDefenceRoll original = make();
		JsonObject json = original.toJsonValue().asObject();
		ReportSolidDefenceRoll restored =
			(ReportSolidDefenceRoll) new ReportSolidDefenceRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getTeamId(), restored.getTeamId());
		assertEquals(original.getAmount(), restored.getAmount());
		assertEquals(original.getRoll(), restored.getRoll());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue().asObject();
		assertEquals("solidDefenceRoll", json.get("reportId").asString());
	}
}
