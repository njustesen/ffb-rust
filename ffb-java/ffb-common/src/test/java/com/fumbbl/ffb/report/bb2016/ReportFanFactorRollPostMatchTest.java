package com.fumbbl.ffb.report.bb2016;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.report.ReportTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportFanFactorRollPostMatchTest {

	private ReportFanFactorRollPostMatch make() {
		return new ReportFanFactorRollPostMatch(new int[]{3, 4}, 1, new int[]{2, 5}, -1);
	}

	@Test
	public void serializationRoundTrip() {
		ReportFanFactorRollPostMatch original = make();
		JsonObject json = original.toJsonValue();
		ReportFanFactorRollPostMatch restored = new ReportFanFactorRollPostMatch().initFrom(ReportTestUtil.source(), json);
		assertArrayEquals(original.getFanFactorRollHome(), restored.getFanFactorRollHome());
		assertEquals(original.getFanFactorModifierHome(), restored.getFanFactorModifierHome());
		assertEquals(original.getFanFactorModifierAway(), restored.getFanFactorModifierAway());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("fanFactorRoll", json.get("reportId").asString());
	}
}
