package com.fumbbl.ffb.report.bb2016;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-model/src/report/bb2016/report_injury.rs tests.
 * InjuryType/SeriousInjury are factory-backed (Rust passes names); left null (options null-safe)
 * while defenderId + armorBroken + rolls + injury PlayerState round-trip.
 */
public class ReportInjuryTest {

	private ReportInjury make() {
		return new ReportInjury("defender1", null, true, null, new int[]{3, 4}, null, new int[]{2, 5}, null,
			null, null, null, new PlayerState(PlayerState.BADLY_HURT), null, "attacker1");
	}

	// rust: serialization_round_trip
	@Test
	public void serializationRoundTrip() {
		ReportInjury original = make();
		JsonObject json = original.toJsonValue();
		ReportInjury restored = new ReportInjury().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getDefenderId(), restored.getDefenderId());
		assertEquals(original.isArmorBroken(), restored.isArmorBroken());
		assertArrayEquals(original.getArmorRoll(), restored.getArmorRoll());
		assertArrayEquals(original.getInjuryRoll(), restored.getInjuryRoll());
		assertNull(restored.getInjuryType());
	}

	// rust: to_json_value_has_report_id
	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("injury", json.get("reportId").asString());
	}
}
