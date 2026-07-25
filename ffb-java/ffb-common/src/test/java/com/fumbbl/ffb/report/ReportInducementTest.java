package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-model/src/report/report_inducement.rs tests.
 * InducementType is factory-backed (Rust passes its name); left null (option is null-safe) while
 * teamId + value round-trip; type resolution is covered by the inducement factory tests.
 */
public class ReportInducementTest {

	private ReportInducement make() {
		return new ReportInducement("team1", null, 150000);
	}

	// rust: serialization_round_trip
	@Test
	public void serializationRoundTrip() {
		ReportInducement original = make();
		JsonObject json = original.toJsonValue();
		ReportInducement restored = new ReportInducement().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getTeamId(), restored.getTeamId());
		assertEquals(original.getValue(), restored.getValue());
		assertNull(restored.getInducementType());
	}

	// rust: to_json_value_has_report_id
	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("inducement", json.get("reportId").asString());
	}
}
