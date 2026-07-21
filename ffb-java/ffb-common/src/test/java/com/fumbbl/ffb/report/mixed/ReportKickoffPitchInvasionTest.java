package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import java.util.Collections;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportKickoffPitchInvasionTest {

	private ReportKickoffPitchInvasion make() {
		return new ReportKickoffPitchInvasion(3, 2, Collections.singletonList("p1"), 1);
	}

	@Test
	public void serializationRoundTrip() {
		ReportKickoffPitchInvasion original = make();
		JsonObject json = (JsonObject) original.toJsonValue();
		ReportKickoffPitchInvasion restored = new ReportKickoffPitchInvasion().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getRollHome(), restored.getRollHome());
		assertEquals(original.getRollAway(), restored.getRollAway());
		assertEquals(original.getAmount(), restored.getAmount());
		assertEquals(original.getAffectedPlayers(), restored.getAffectedPlayers());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = (JsonObject) make().toJsonValue();
		assertEquals("kickoffPitchInvasion", json.get("reportId").asString());
	}
}
