package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.inducement.InducementType;
import com.fumbbl.ffb.inducement.Usage;
import com.fumbbl.ffb.report.ReportInducement;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class InducementMessageTest extends ReportMessageTestBase {

	@Test
	public void rendersExtraTeamTrainingSingular() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamHome().getName()).willReturn("Team home");

		InducementType type = new InducementType("extraTeamTraining", "Extra Team Training", "Re-Roll", "Re-Rolls",
			null, null, Usage.REROLL);
		ReportInducement report = new ReportInducement("home", type, 1);
		List<Run> runs = render(new InducementMessage(), report);

		assertEquals("Team home", runs.get(0).text);
		assertEquals(" use ", runs.get(1).text);
		assertEquals("Extra Team Training", runs.get(2).text);
		assertEquals(" to add 1 Re-Roll.", runs.get(3).text);
	}

	@Test
	public void rendersWanderingApothecariesPlural() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamAway().getName()).willReturn("Team away");

		InducementType type = new InducementType("wanderingApothecaries", "Wandering Apothecaries", "Apothecary",
			"Apothecaries", null, null, Usage.APOTHECARY);
		ReportInducement report = new ReportInducement("away", type, 2);
		List<Run> runs = render(new InducementMessage(), report);

		assertEquals("Team away", runs.get(0).text);
		assertEquals("Wandering Apothecaries", runs.get(2).text);
		assertEquals(" to add 2 Apothecaries.", runs.get(3).text);
	}

	@Test
	public void rendersRegenerationBranchForKnownTypes() {
		// render() re-verifies ALL invocations captured on the shared `log` mock since the
		// test method started, so each loop iteration's runs are APPENDED to the previous
		// iterations' runs rather than replacing them (the mock/statusReport are not reset
		// between iterations - only @BeforeEach resets them, which runs once per test method).
		// Each iteration emits exactly 5 runs (team name, " use ", description, real status
		// text, null println-terminator), so iteration i's runs start at offset i*5.
		String[] typeNames = { "igor", "mortuaryAssistant", "plagueDoctor" };
		for (int i = 0; i < typeNames.length; i++) {
			String typeName = typeNames[i];
			given(game.getTeamHome().getId()).willReturn("home");
			given(game.getTeamHome().getName()).willReturn("Team home");

			// The description is set to the raw type name here so the assertion mirrors
			// the Rust test's expectation. In the real game data these InducementType
			// instances carry human-readable descriptions (e.g. "Igor"); the Java
			// handler always prints `getDescription()`, never the raw type name.
			InducementType type = new InducementType(typeName, typeName, typeName, typeName, null, null,
				Usage.REGENERATION);
			ReportInducement report = new ReportInducement("home", type, 1);
			List<Run> runs = render(new InducementMessage(), report);

			int base = i * 5;
			assertEquals(typeName, runs.get(base + 2).text);
			assertEquals(" to re-roll the failed Regeneration.", runs.get(base + 3).text);
		}
	}

	@Test
	public void rendersNothingWhenTeamIdEmpty() {
		InducementType type = new InducementType("extraTeamTraining", "Extra Team Training", "Re-Roll", "Re-Rolls",
			null, null, Usage.REROLL);
		ReportInducement report = new ReportInducement("", type, 1);
		List<Run> runs = render(new InducementMessage(), report);

		assertTrue(runs.isEmpty());
	}

	@Test
	public void rendersNothingForUnknownInducementType() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamHome().getName()).willReturn("Team home");

		InducementType type = new InducementType("bribery", "Bribery", "Bribe", "Bribes", null, null,
			Usage.UNSPECIFIC);
		ReportInducement report = new ReportInducement("home", type, 1);
		List<Run> runs = render(new InducementMessage(), report);

		assertEquals(1, runs.size()); // team name only, no usage branch matched
	}
}
