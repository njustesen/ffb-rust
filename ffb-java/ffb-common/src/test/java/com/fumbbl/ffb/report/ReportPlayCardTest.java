package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.inducement.Card;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-model/src/report/report_play_card.rs tests.
 * Java {@code Card} is factory-backed (Rust passes its name); left null (CARD option null-safe)
 * while teamId + catcher playerId round-trip.
 */
public class ReportPlayCardTest {

	private ReportPlayCard make() {
		return new ReportPlayCard("team1", (Card) null, "p1");
	}

	// rust: serialization_round_trip
	@Test
	public void serializationRoundTrip() {
		ReportPlayCard original = make();
		JsonObject json = original.toJsonValue();
		ReportPlayCard restored = new ReportPlayCard().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getTeamId(), restored.getTeamId());
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertNull(restored.getCard());
	}

	// rust: to_json_value_has_report_id
	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("playCard", json.get("reportId").asString());
	}
}
