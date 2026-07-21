package com.fumbbl.ffb.report.bb2016;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.factory.KickoffResultFactory;
import com.fumbbl.ffb.kickoff.KickoffResult;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportKickoffExtraReRollTest {

	private KickoffResult kickoffResult(String name) {
		KickoffResultFactory factory = ReportTestUtil.source().getFactory(FactoryType.Factory.KICKOFF_RESULT);
		return factory.forName(name);
	}

	private ReportKickoffExtraReRoll make() {
		return new ReportKickoffExtraReRoll(kickoffResult("Brilliant Coaching"), 3, true, 2, false);
	}

	@Test
	public void serializationRoundTrip() {
		ReportKickoffExtraReRoll original = make();
		JsonObject json = original.toJsonValue();
		ReportKickoffExtraReRoll restored = new ReportKickoffExtraReRoll().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getRollHome(), restored.getRollHome());
		assertEquals(original.getRollAway(), restored.getRollAway());
		assertEquals(original.isHomeGainsReRoll(), restored.isHomeGainsReRoll());
		assertEquals(original.getKickoffResult(), restored.getKickoffResult());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("extraReRoll", json.get("reportId").asString());
	}
}
