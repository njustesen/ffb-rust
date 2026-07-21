package com.fumbbl.ffb.report.bb2016;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportRefereeTest {

	private ReportReferee make() {
		return new ReportReferee(true);
	}

	@Test
	public void serializationRoundTrip() {
		ReportReferee original = make();
		JsonObject json = original.toJsonValue();
		ReportReferee restored = new ReportReferee().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.isFoulingPlayerBanned(), restored.isFoulingPlayerBanned());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("referee", json.get("reportId").asString());
	}
}
