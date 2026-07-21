package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportRiotousRookiesTest {

	private ReportRiotousRookies make() {
		return new ReportRiotousRookies(new int[]{2, 3}, 1, "team1");
	}

	@Test
	public void serializationRoundTrip() {
		ReportRiotousRookies original = make();
		JsonObject json = original.toJsonValue();
		ReportRiotousRookies restored = new ReportRiotousRookies().initFrom(ReportTestUtil.source(), json);
		assertArrayEquals(original.getRoll(), restored.getRoll());
		assertEquals(original.getAmount(), restored.getAmount());
		assertEquals(original.getTeamId(), restored.getTeamId());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("riotousRookies", json.get("reportId").asString());
	}
}
