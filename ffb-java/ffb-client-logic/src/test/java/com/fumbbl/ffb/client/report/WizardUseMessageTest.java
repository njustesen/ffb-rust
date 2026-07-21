package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.SpecialEffect;
import com.fumbbl.ffb.report.ReportWizardUse;
import org.junit.jupiter.api.Test;

import java.util.List;
import java.util.stream.Collectors;

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class WizardUseMessageTest extends ReportMessageTestBase {

	private void givenTeams() {
		given(game.getTeamHome().getId()).willReturn("home");
		given(game.getTeamHome().getName()).willReturn("Home Wizards");
		given(game.getTeamAway().getName()).willReturn("Away Wizards");
	}

	@Test
	public void homeTeamLightning() {
		givenTeams();

		ReportWizardUse report = new ReportWizardUse("home", SpecialEffect.LIGHTNING);
		List<Run> runs = render(new WizardUseMessage(), report);

		List<String> texts = runs.stream().map(r -> r.text).collect(Collectors.toList());
		assertTrue(texts.contains("Home Wizards"));
		assertTrue(texts.contains(" casts a Lightning spell."));
	}

	@Test
	public void awayTeamZap() {
		givenTeams();

		ReportWizardUse report = new ReportWizardUse("away", SpecialEffect.ZAP);
		List<Run> runs = render(new WizardUseMessage(), report);

		List<String> texts = runs.stream().map(r -> r.text).collect(Collectors.toList());
		assertTrue(texts.contains("Away Wizards"));
		assertTrue(texts.contains(" casts a Zap! spell."));
	}

	@Test
	public void fireballDefault() {
		givenTeams();

		ReportWizardUse report = new ReportWizardUse("home", SpecialEffect.FIREBALL);
		List<Run> runs = render(new WizardUseMessage(), report);

		List<String> texts = runs.stream().map(r -> r.text).collect(Collectors.toList());
		assertTrue(texts.contains(" casts a Fireball spell."));
	}

	@Test
	public void unknownTeamIdFallsThroughToAway() {
		givenTeams();

		ReportWizardUse report = new ReportWizardUse("nonexistent", SpecialEffect.FIREBALL);
		List<Run> runs = render(new WizardUseMessage(), report);

		List<String> texts = runs.stream().map(r -> r.text).collect(Collectors.toList());
		assertTrue(texts.contains("Away Wizards"));
	}
}
