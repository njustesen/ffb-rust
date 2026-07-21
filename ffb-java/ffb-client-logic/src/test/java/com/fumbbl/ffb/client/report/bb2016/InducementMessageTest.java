package com.fumbbl.ffb.client.report.bb2016;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.inducement.InducementType;
import com.fumbbl.ffb.inducement.Usage;
import com.fumbbl.ffb.report.ReportInducement;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class InducementMessageTest extends ReportMessageTestBase {

	@Mock
	private InducementType inducementType;

	@Test
	public void getKeyIsInducement() {
		assertEquals("inducement", new InducementMessage().getKey());
	}

	@Test
	public void extraTeamTrainingPluralRerolls() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(inducementType.hasUsage(Usage.REROLL)).willReturn(true);

		ReportInducement report = new ReportInducement("home", inducementType, 2);
		List<Run> runs = render(new InducementMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " to add 2 Re-Rolls.".equals(r.text)));
	}

	@Test
	public void wanderingApothecariesSingular() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(inducementType.hasUsage(Usage.APOTHECARY)).willReturn(true);

		ReportInducement report = new ReportInducement("away", inducementType, 1);
		List<Run> runs = render(new InducementMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " to add 1 Apothecary.".equals(r.text)));
	}

	@Test
	public void igorMessage() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(inducementType.hasUsage(Usage.REGENERATION)).willReturn(true);

		ReportInducement report = new ReportInducement("home", inducementType, 1);
		List<Run> runs = render(new InducementMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " to re-roll the failed Regeneration.".equals(r.text)));
	}

	@Test
	public void emptyTeamIdProducesNoOutput() {
		ReportInducement report = new ReportInducement("", inducementType, 1);
		List<Run> runs = render(new InducementMessage(), report);

		assertTrue(runs.isEmpty());
	}
}
