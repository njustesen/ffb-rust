package com.fumbbl.ffb.client.report.bb2016;

import com.fumbbl.ffb.FactoryType.Factory;
import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.factory.INamedObjectFactory;
import com.fumbbl.ffb.mechanics.Mechanic;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.modifiers.RollModifier;
import com.fumbbl.ffb.report.ReportSkillRoll;
import com.fumbbl.ffb.report.bb2016.ReportHypnoticGazeRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.BDDMockito.given;

class HypnoticGazeRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player gazer;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player victim;

	@SuppressWarnings("rawtypes")
	@Mock
	private INamedObjectFactory mechanicFactory;

	@Mock
	private com.fumbbl.ffb.mechanics.bb2016.AgilityMechanic agilityMechanic;

	private void stubAgilityMechanic(String result) {
		given(game.getRules().<INamedObjectFactory>getFactory(Factory.MECHANIC)).willReturn(mechanicFactory);
		given(mechanicFactory.forName(Mechanic.Type.AGILITY.name())).willReturn(agilityMechanic);
		given(agilityMechanic.formatHypnoticGazeResult(any(ReportSkillRoll.class), any())).willReturn(result);
	}

	@Test
	public void getKeyIsHypnoticGazeRoll() {
		assertEquals("hypnoticGazeRoll", new HypnoticGazeRollMessage().getKey());
	}

	@Test
	public void successfulFirstRollPrintsIntroAndHypnotizes() {
		given(game.getActingPlayer().getPlayer()).willReturn(gazer);
		given(game.getTeamHome().hasPlayer(gazer)).willReturn(true);
		given(gazer.getName()).willReturn("Gazer");
		given(gazer.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getDefender()).willReturn(victim);
		given(victim.getName()).willReturn("Victim");
		stubAgilityMechanic(" (AG 4 + Roll > 6).");

		ReportHypnoticGazeRoll report = new ReportHypnoticGazeRoll(null, true, 5, 2, false, new RollModifier<?>[0]);
		List<Run> runs = render(new HypnoticGazeRollMessage(), report);

		assertEquals("Gazer", runs.get(0).text);
		assertEquals(1, runs.stream().filter(r -> " hypnotizes his victim.".equals(r.text)).count());
	}

	@Test
	public void reRolledSkipsIntroLine() {
		given(game.getActingPlayer().getPlayer()).willReturn(gazer);
		given(game.getTeamHome().hasPlayer(gazer)).willReturn(true);
		given(gazer.getName()).willReturn("Gazer");
		given(gazer.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getDefender()).willReturn(victim);

		ReportHypnoticGazeRoll report = new ReportHypnoticGazeRoll(null, false, 1, 2, true, new RollModifier<?>[0]);
		List<Run> runs = render(new HypnoticGazeRollMessage(), report);

		assertEquals("Hypnotic Gaze Roll [ 1 ]", runs.get(0).text);
	}

	@Test
	public void failedRollReportsNeededRollWithAgility() {
		given(game.getActingPlayer().getPlayer()).willReturn(gazer);
		given(game.getTeamHome().hasPlayer(gazer)).willReturn(true);
		given(gazer.getName()).willReturn("Gazer");
		given(gazer.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getDefender()).willReturn(victim);
		stubAgilityMechanic(" (AG 4 + Roll > 6).");

		ReportHypnoticGazeRoll report = new ReportHypnoticGazeRoll(null, false, 1, 2, false, new RollModifier<?>[0]);
		List<Run> runs = render(new HypnoticGazeRollMessage(), report);

		Run needed = runs.stream().filter(r -> r.textStyle == TextStyle.NEEDED_ROLL).findFirst().orElseThrow();
		assertEquals("Roll a 2+ to succeed (AG 4 + Roll > 6).", needed.text);
	}

	@Test
	public void failedRollReportsNeededRollWithAgilityAndModifier() {
		given(game.getActingPlayer().getPlayer()).willReturn(gazer);
		given(game.getTeamHome().hasPlayer(gazer)).willReturn(true);
		given(gazer.getName()).willReturn("Gazer");
		given(gazer.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getDefender()).willReturn(victim);
		stubAgilityMechanic(" (AG 4 - Foo + Roll > 6).");

		ReportHypnoticGazeRoll report = new ReportHypnoticGazeRoll(null, false, 1, 2, false, new RollModifier<?>[0]);
		List<Run> runs = render(new HypnoticGazeRollMessage(), report);

		Run needed = runs.stream().filter(r -> r.textStyle == TextStyle.NEEDED_ROLL).findFirst().orElseThrow();
		assertEquals("Roll a 2+ to succeed (AG 4 - Foo + Roll > 6).", needed.text);
	}
}
