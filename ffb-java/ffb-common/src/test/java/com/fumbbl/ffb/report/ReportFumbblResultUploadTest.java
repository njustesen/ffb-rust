package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportFumbblResultUploadTest {

	private ReportFumbblResultUpload make() {
		return new ReportFumbblResultUpload(true, "OK");
	}

	@Test
	public void serializationRoundTrip() {
		ReportFumbblResultUpload original = make();
		JsonObject json = original.toJsonValue();
		ReportFumbblResultUpload restored = new ReportFumbblResultUpload().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.isSuccessful(), restored.isSuccessful());
		assertEquals(original.getUploadStatus(), restored.getUploadStatus());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("fumbblResultUpload", json.get("reportId").asString());
	}
}
