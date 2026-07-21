package com.fumbbl.ffb.report.bb2016;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.HeatExhaustion;
import com.fumbbl.ffb.KnockoutRecovery;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import java.util.ArrayList;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportTurnEndTest {

	private ReportTurnEnd make() {
		return new ReportTurnEnd("scorer", new KnockoutRecovery[0], new HeatExhaustion[0], new ArrayList<>());
	}

	@Test
	public void serializationRoundTrip() {
		ReportTurnEnd original = make();
		JsonObject json = original.toJsonValue();
		ReportTurnEnd restored = new ReportTurnEnd().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerIdTouchdown(), restored.getPlayerIdTouchdown());
		assertEquals(original.getUnzappedPlayers().size(), restored.getUnzappedPlayers().size());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("turnEnd", json.get("reportId").asString());
	}
}
