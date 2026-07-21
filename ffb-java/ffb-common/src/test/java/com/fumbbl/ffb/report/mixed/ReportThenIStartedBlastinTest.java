package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportThenIStartedBlastinTest {

	private ReportThenIStartedBlastin make() {
		return new ReportThenIStartedBlastin("p1", "t1", 4, true, false);
	}

	@Test
	public void serializationRoundTrip() {
		ReportThenIStartedBlastin original = make();
		JsonObject json = original.toJsonValue().asObject();
		ReportThenIStartedBlastin restored = new ReportThenIStartedBlastin().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.getTargetPlayerId(), restored.getTargetPlayerId());
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.isSuccess(), restored.isSuccess());
		assertEquals(original.isFumble(), restored.isFumble());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue().asObject();
		assertEquals("thenIStartedBlastin", json.get("reportId").asString());
	}
}
