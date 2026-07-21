package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportBombExplodesAfterCatchTest {

	private ReportBombExplodesAfterCatch make() {
		return new ReportBombExplodesAfterCatch("p1", true, 5);
	}

	@Test
	public void serializationRoundTrip() {
		ReportBombExplodesAfterCatch original = make();
		JsonObject json = (JsonObject) original.toJsonValue();
		ReportBombExplodesAfterCatch restored = new ReportBombExplodesAfterCatch().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getCatcherId(), restored.getCatcherId());
		assertEquals(original.explodes(), restored.explodes());
		assertEquals(original.getRoll(), restored.getRoll());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = (JsonObject) make().toJsonValue();
		assertEquals("bombExplodesAfterCatch", json.get("reportId").asString());
	}
}
