package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportAnimalSavageryTest {

	private ReportAnimalSavagery make() {
		return new ReportAnimalSavagery("a1", "d1");
	}

	@Test
	public void serializationRoundTrip() {
		ReportAnimalSavagery original = make();
		JsonObject json = (JsonObject) original.toJsonValue();
		ReportAnimalSavagery restored = new ReportAnimalSavagery().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getAttackerId(), restored.getAttackerId());
		assertEquals(original.getDefenderId(), restored.getDefenderId());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = (JsonObject) make().toJsonValue();
		assertEquals("animalSavagery", json.get("reportId").asString());
	}
}
