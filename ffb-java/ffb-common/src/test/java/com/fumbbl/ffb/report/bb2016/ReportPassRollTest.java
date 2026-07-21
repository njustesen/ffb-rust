package com.fumbbl.ffb.report.bb2016;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.PassingDistance;
import com.fumbbl.ffb.mechanics.PassResult;
import com.fumbbl.ffb.modifiers.PassModifier;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportPassRollTest {

	private ReportPassRoll make() {
		return new ReportPassRoll("p1", 4, 3, false, new PassModifier[0], PassingDistance.LONG_PASS, false,
			PassResult.ACCURATE, false);
	}

	@Test
	public void serializationRoundTrip() {
		ReportPassRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportPassRoll restored = new ReportPassRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getPlayerId(), restored.getPlayerId());
		assertEquals(original.isSuccessful(), restored.isSuccessful());
		assertEquals(original.getPassingDistance(), restored.getPassingDistance());
		assertEquals(original.getResult(), restored.getResult());
		assertEquals(original.isHailMaryPass(), restored.isHailMaryPass());
		assertEquals(original.isBomb(), restored.isBomb());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("passRoll", json.get("reportId").asString());
	}
}
