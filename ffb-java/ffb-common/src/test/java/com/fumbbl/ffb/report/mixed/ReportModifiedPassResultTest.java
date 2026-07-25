package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.mechanics.PassResult;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-model/src/report/report_modified_pass_result.rs tests.
 * Skill is factory-backed (left null, option null-safe); PassResult is a plain enum and round-trips.
 */
public class ReportModifiedPassResultTest {

	private ReportModifiedPassResult make() {
		return new ReportModifiedPassResult(null, PassResult.ACCURATE);
	}

	// rust: serialization_round_trip
	@Test
	public void serializationRoundTrip() {
		ReportModifiedPassResult original = make();
		JsonObject json = original.toJsonValue();
		ReportModifiedPassResult restored = new ReportModifiedPassResult().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPassResult(), restored.getPassResult());
		assertNull(restored.getSkill());
	}

	// rust: to_json_value_has_report_id
	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("modifiedPassResult", json.get("reportId").asString());
	}
}
