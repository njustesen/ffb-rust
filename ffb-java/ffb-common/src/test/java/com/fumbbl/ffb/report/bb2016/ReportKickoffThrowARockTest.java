package com.fumbbl.ffb.report.bb2016;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportKickoffThrowARockTest {

	private ReportKickoffThrowARock make() {
		return new ReportKickoffThrowARock(4, 2, new String[]{"p1", "p2"});
	}

	@Test
	public void serializationRoundTrip() {
		ReportKickoffThrowARock original = make();
		JsonObject json = original.toJsonValue();
		ReportKickoffThrowARock restored = new ReportKickoffThrowARock().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getRollHome(), restored.getRollHome());
		assertEquals(original.getRollAway(), restored.getRollAway());
		assertArrayEquals(original.getPlayersHit(), restored.getPlayersHit());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("kickoffThrowARock", json.get("reportId").asString());
	}
}
