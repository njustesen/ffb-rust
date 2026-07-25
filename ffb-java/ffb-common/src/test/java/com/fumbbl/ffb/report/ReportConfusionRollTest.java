package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-model/src/report/report_confusion_roll.rs tests.
 */
public class ReportConfusionRollTest {

	private ReportConfusionRoll make() {
		return new ReportConfusionRoll("p1", true, 4, 2, false, null);
	}

	// rust: serialization_round_trip
	@Test
	public void serializationRoundTrip() {
		ReportConfusionRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportConfusionRoll restored = new ReportConfusionRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
		assertEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.getMinimumRoll(), restored.getMinimumRoll());
		assertEquals(original.isReRolled(), restored.isReRolled());
		assertEquals(original.getConfusionSkill(), restored.getConfusionSkill());
	}

	// rust: to_json_value_has_report_id
	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("confusionRoll", json.get("reportId").asString());
	}
}
