package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-model/src/report/report_kickoff_result.rs tests.
 *
 * <p>Java {@code KickoffResult} is a factory-backed edition interface (not a plain enum like Rust),
 * so the result object is left null here and the round-trip exercises the kickoff roll + reportId;
 * the KickoffResult factory round-trip is covered by the kickoff mapping/factory tests.
 */
public class ReportKickoffResultTest {

	private ReportKickoffResult make() {
		return new ReportKickoffResult(null, new int[]{3, 4});
	}

	// rust: serialization_round_trip
	@Test
	public void serializationRoundTrip() {
		ReportKickoffResult original = make();
		JsonObject json = original.toJsonValue();
		ReportKickoffResult restored = new ReportKickoffResult().initFrom(ReportTestUtil.source(), json);
		assertArrayEquals(original.getKickoffRoll(), restored.getKickoffRoll());
		assertNull(restored.getKickoffResult());
	}

	// rust: to_json_value_has_report_id
	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("kickoffResult", json.get("reportId").asString());
	}
}
