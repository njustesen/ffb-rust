package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import com.fumbbl.ffb.report.logcontrol.SkipInjuryParts;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-model/src/report/mixed/report_injury.rs tests.
 * Factory-backed fields (InjuryType/SeriousInjury/CasualtyModifier/SkipInjuryParts) left null
 * (options null-safe); defenderId + attackerId + armorBroken + rolls round-trip.
 */
public class ReportInjuryTest {

	private ReportInjury make() {
		return new ReportInjury("d1", null, true, null, new int[]{3, 4}, null, new int[]{5}, null,
			null, null, null, null, null, "a1", null, null, SkipInjuryParts.NONE);
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
