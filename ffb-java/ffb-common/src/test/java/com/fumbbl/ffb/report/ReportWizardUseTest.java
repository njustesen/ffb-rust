package com.fumbbl.ffb.report;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.SpecialEffect;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class ReportWizardUseTest {

	private ReportWizardUse make() {
		return new ReportWizardUse("team1", SpecialEffect.FIREBALL);
	}

	@Test
	public void serializationRoundTrip() {
		ReportWizardUse original = make();
		JsonObject json = original.toJsonValue();
		ReportWizardUse restored = new ReportWizardUse().initFrom(ReportTestUtil.source(), json);
		assertEquals(original.getTeamId(), restored.getTeamId());
		assertEquals(original.getWizardSpell(), restored.getWizardSpell());
	}

	@Test
	public void toJsonValueHasReportId() {
		JsonObject json = make().toJsonValue();
		assertEquals("wizardUse", json.get("reportId").asString());
	}
}
