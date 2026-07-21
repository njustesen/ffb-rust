package com.fumbbl.ffb.report.mixed;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportBlockReRollTest {

	private ReportBlockReRoll make() {
		return new ReportBlockReRoll(new int[]{2, 5}, "p1", null);
	}

	@Test
	public void serializationRoundTrip() {
		ReportBlockReRoll original = make();
		JsonObject json = (JsonObject) original.toJsonValue();
		ReportBlockReRoll restored = new ReportBlockReRoll().initFrom(ReportTestUtil.source(), json);
		assertArrayEquals(original.getBlockRoll(), restored.getBlockRoll());
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.getReRollSource(), restored.getReRollSource());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = (JsonObject) make().toJsonValue();
		assertEquals("blockReRoll", json.get("reportId").asString());
	}
}
