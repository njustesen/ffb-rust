package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.inducement.Card;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-model/src/report/report_card_deactivated.rs tests.
 *
 * <p>Java {@code Card} is a factory-backed object (Rust passes the card name); the CARD json option
 * is null-safe, so the null card round-trips as null and the test exercises reportId + the null
 * round-trip. Card resolution is covered by the inducement/card factory tests.
 */
public class ReportCardDeactivatedTest {

	private ReportCardDeactivated make() {
		return new ReportCardDeactivated((Card) null);
	}

	// rust: serialization_round_trip
	@Test
	public void serializationRoundTrip() {
		ReportCardDeactivated original = make();
		JsonObject json = original.toJsonValue();
		ReportCardDeactivated restored = new ReportCardDeactivated().initFrom(ReportTestUtil.source(), json);
		assertNull(restored.getCard());
	}

	// rust: to_json_value_has_report_id
	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("cardDeactivated", json.get("reportId").asString());
	}
}
