package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-model/src/report/report_modified_dodge_result_successful.rs tests.
 * Skill is factory-backed (left null, option null-safe); the test exercises the reportId + null
 * round-trip.
 */
public class ReportModifiedDodgeResultSuccessfulTest {

	private ReportModifiedDodgeResultSuccessful make() {
		return new ReportModifiedDodgeResultSuccessful((Skill) null);
	}

	// rust: serialization_round_trip
	@Test
	public void serializationRoundTrip() {
		ReportModifiedDodgeResultSuccessful original = make();
		JsonObject json = original.toJsonValue();
		ReportModifiedDodgeResultSuccessful restored =
			new ReportModifiedDodgeResultSuccessful().initFrom(ReportTestUtil.source(), json);
		assertNull(restored.getSkill());
	}

	// rust: to_json_value_has_report_id
	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("modifiedDodgeResultSuccessful", json.get("reportId").asString());
	}
}
