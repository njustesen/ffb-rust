package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportKickoffSequenceActivationsCountTest {

	private ReportKickoffSequenceActivationsCount make() {
		return new ReportKickoffSequenceActivationsCount(5, 2, 3);
	}

	@Test
	public void serializationRoundTrip() {
		ReportKickoffSequenceActivationsCount original = make();
		JsonObject json = (JsonObject) original.toJsonValue();
		ReportKickoffSequenceActivationsCount restored = (ReportKickoffSequenceActivationsCount) new ReportKickoffSequenceActivationsCount().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getAmount(), restored.getAmount());
		assertEquals(original.getAvailable(), restored.getAvailable());
		assertEquals(original.getLimit(), restored.getLimit());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = (JsonObject) make().toJsonValue();
		assertEquals("kickoffSequenceActivationsCount", json.get("reportId").asString());
	}
}
