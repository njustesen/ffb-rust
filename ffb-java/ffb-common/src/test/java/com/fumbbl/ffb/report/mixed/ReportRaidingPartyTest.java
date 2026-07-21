package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportRaidingPartyTest {

	private ReportRaidingParty make() {
		return new ReportRaidingParty("p1", "p2", Direction.NORTHEAST);
	}

	@Test
	public void serializationRoundTrip() {
		ReportRaidingParty original = make();
		JsonObject json = original.toJsonValue().asObject();
		ReportRaidingParty restored = new ReportRaidingParty().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.getOtherPlayerId(), restored.getOtherPlayerId());
		assertEquals(original.getDirection(), restored.getDirection());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue().asObject();
		assertEquals("raidingParty", json.get("reportId").asString());
	}
}
