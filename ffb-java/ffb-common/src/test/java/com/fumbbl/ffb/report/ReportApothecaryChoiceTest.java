package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.PlayerState;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-model/src/report/report_apothecary_choice.rs tests.
 *
 * <p>Java {@code SeriousInjury} is a factory-backed edition object (Rust passes its name); the
 * SERIOUS_INJURY json option is null-safe, so it is left null here while the playerId + playerState
 * round-trip is exercised. SeriousInjury resolution is covered by the injury/serious-injury tests.
 */
public class ReportApothecaryChoiceTest {

	private ReportApothecaryChoice make() {
		return new ReportApothecaryChoice("p1", new PlayerState(PlayerState.STANDING), null);
	}

	// rust: serialization_round_trip
	@Test
	public void serializationRoundTrip() {
		ReportApothecaryChoice original = make();
		JsonObject json = original.toJsonValue();
		ReportApothecaryChoice restored = new ReportApothecaryChoice().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.getPlayerState(), restored.getPlayerState());
		assertNull(restored.getSeriousInjury());
	}

	// rust: to_json_value_has_report_id
	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("apothecaryChoice", json.get("reportId").asString());
	}
}
