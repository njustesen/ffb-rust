package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.HeatExhaustion;
import com.fumbbl.ffb.KnockoutRecovery;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import java.util.Collections;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

public class ReportTurnEndTest {

	private ReportTurnEnd make() {
		List<Player<?>> unzapped = Collections.emptyList();
		return new ReportTurnEnd("scorer",
			new KnockoutRecovery[]{new KnockoutRecovery("ko1", true, 0, 0, null)},
			new HeatExhaustion[]{new HeatExhaustion("he1", true, 3)},
			unzapped, 5);
	}

	@Test
	public void serializationRoundTrip() {
		ReportTurnEnd original = make();
		JsonObject json = original.toJsonValue().asObject();
		ReportTurnEnd restored = new ReportTurnEnd().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerIdTouchdown(), restored.getPlayerIdTouchdown());
		assertEquals(1, restored.getKnockoutRecoveries().length);
		assertEquals("ko1", restored.getKnockoutRecoveries()[0].getPlayerId());
		assertTrue(restored.getKnockoutRecoveries()[0].isRecovering());
		assertEquals(1, restored.getHeatExhaustions().length);
		assertEquals("he1", restored.getHeatExhaustions()[0].getPlayerId());
		assertEquals(3, restored.getHeatExhaustions()[0].getRoll());
		assertTrue(restored.getUnzappedPlayers().isEmpty());
		assertEquals(original.getHeatRoll(), restored.getHeatRoll());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue().asObject();
		assertEquals("turnEnd", json.get("reportId").asString());
	}
}
