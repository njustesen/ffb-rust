package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportWinningsTest {

	private ReportWinnings make() {
		return new ReportWinnings(50000, 30000);
	}

	@Test
	public void serializationRoundTrip() {
		ReportWinnings original = make();
		JsonObject json = original.toJsonValue().asObject();
		ReportWinnings restored = new ReportWinnings().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getWinningsHome(), restored.getWinningsHome());
		assertEquals(original.getWinningsAway(), restored.getWinningsAway());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue().asObject();
		assertEquals("winnings", json.get("reportId").asString());
	}
}
