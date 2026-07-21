package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportProjectileVomitTest {

	private ReportProjectileVomit make() {
		return new ReportProjectileVomit("p1", true, 4, 2, false, "d1");
	}

	@Test
	public void serializationRoundTrip() {
		ReportProjectileVomit original = make();
		JsonObject json = original.toJsonValue().asObject();
		ReportProjectileVomit restored = new ReportProjectileVomit().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.getMinimumRoll(), restored.getMinimumRoll());
		assertEquals(original.isReRolled(), restored.isReRolled());
		assertEquals(original.getDefenderId(), restored.getDefenderId());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue().asObject();
		assertEquals("projectileVomit", json.get("reportId").asString());
	}
}
