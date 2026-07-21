package com.fumbbl.ffb.report.bb2016;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportKickTeamMateRollTest {

	private ReportKickTeamMateRoll make() {
		return new ReportKickTeamMateRoll("kicker", "kicked", true, new int[]{3, 4}, false, 3);
	}

	@Test
	public void serializationRoundTrip() {
		ReportKickTeamMateRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportKickTeamMateRoll restored = new ReportKickTeamMateRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getKickingPlayerId(), restored.getKickingPlayerId());
		assertEquals(original.getKickedPlayerId(), restored.getKickedPlayerId());
		assertEquals(original.getKickDistance(), restored.getKickDistance());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
		assertArrayEquals(original.getRoll(), restored.getRoll());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("kickTeamMateRoll", json.get("reportId").asString());
	}
}
