package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.inducement.bb2025.Prayer;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-model/src/report/report_prayer_end.rs tests.
 * A real Prayer (edition enum) is required: toJson writes prayer.getName() (NPE on null).
 *
 * <p>NOTE (test equalization): the Rust serialization_round_trip test is fixture-inexpressible here.
 * ReportPrayerEnd.initFrom resolves the prayer via source.getFactory(PRAYER).forName(...), but none
 * of the ffb-common test sources (ReportTestUtil.source(), NetCommandTestUtil.gameSource()/
 * applicationSource()) has a populated PRAYER factory — getFactory(PRAYER) returns null and NPEs.
 * Populating it needs an explicit PrayerFactory.initialize(game) wired into a custom IFactorySource,
 * out of scope. Left Rust-only; the reportId direction is ported below.
 */
public class ReportPrayerEndTest {

	private ReportPrayerEnd make() {
		return new ReportPrayerEnd(Prayer.DAZZLING_CATCHING);
	}

	// rust: to_json_value_has_report_id
	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("prayerEnd", json.get("reportId").asString());
	}
}
