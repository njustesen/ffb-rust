package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.inducement.BriberyAndCorruptionAction;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportBriberyAndCorruptionReRollTest {

	private ReportBriberyAndCorruptionReRoll make() {
		return new ReportBriberyAndCorruptionReRoll("team1", BriberyAndCorruptionAction.USED);
	}

	@Test
	public void serializationRoundTrip() {
		ReportBriberyAndCorruptionReRoll original = make();
		JsonObject json = (JsonObject) original.toJsonValue();
		ReportBriberyAndCorruptionReRoll restored = new ReportBriberyAndCorruptionReRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getTeamId(), restored.getTeamId());
		assertEquals(original.getAction(), restored.getAction());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = (JsonObject) make().toJsonValue();
		assertEquals("briberyAndCorruptionReRoll", json.get("reportId").asString());
	}
}
