package com.fumbbl.ffb.report.bb2016;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportKickoffPitchInvasionTest {

	private ReportKickoffPitchInvasion make() {
		return new ReportKickoffPitchInvasion(new int[]{4, 3}, new boolean[]{true, false}, new int[]{2, 5},
			new boolean[]{false, true});
	}

	@Test
	public void serializationRoundTrip() {
		ReportKickoffPitchInvasion original = make();
		JsonObject json = original.toJsonValue();
		ReportKickoffPitchInvasion restored = new ReportKickoffPitchInvasion().initFrom(ReportTestUtil.source(), json);
		assertArrayEquals(original.getRollsHome(), restored.getRollsHome());
		assertArrayEquals(original.getPlayersAffectedHome(), restored.getPlayersAffectedHome());
		assertArrayEquals(original.getRollsAway(), restored.getRollsAway());
		assertArrayEquals(original.getPlayersAffectedAway(), restored.getPlayersAffectedAway());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("kickoffPitchInvasion", json.get("reportId").asString());
	}
}
