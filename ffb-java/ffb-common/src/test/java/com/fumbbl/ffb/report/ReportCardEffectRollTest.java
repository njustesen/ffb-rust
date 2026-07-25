package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.inducement.Card;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-model/src/report/report_card_effect_roll.rs tests.
 * Card and CardEffect are factory-backed (Rust passes their names); left null (options null-safe /
 * card-effect guarded) while the roll round-trips.
 */
public class ReportCardEffectRollTest {

	private ReportCardEffectRoll make() {
		return new ReportCardEffectRoll((Card) null, 3);
	}

	// rust: serialization_round_trip
	@Test
	public void serializationRoundTrip() {
		ReportCardEffectRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportCardEffectRoll restored = new ReportCardEffectRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getRoll(), restored.getRoll());
		assertNull(restored.getCard());
		assertNull(restored.getCardEffect());
	}

	// rust: to_json_value_has_report_id
	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("cardEffectRoll", json.get("reportId").asString());
	}
}
