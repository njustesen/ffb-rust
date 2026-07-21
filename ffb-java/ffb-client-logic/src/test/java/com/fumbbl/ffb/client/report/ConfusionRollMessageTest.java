package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.report.ReportConfusionRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class ConfusionRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Mock
	private Skill confusionSkill;

	private void givenActingPlayer() {
		given(game.getActingPlayer().getPlayer()).willReturn(player);
		given(player.getName()).willReturn("Confused");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
	}

	@Test
	public void noConfusionSkillRendersNothing() {
		ReportConfusionRoll report = new ReportConfusionRoll("p1", true, 4, 2, false, null);
		List<Run> runs = render(new ConfusionRollMessage(), report);
		assertTrue(runs.isEmpty());
	}

	@Test
	public void successfulRollActsNormally() {
		givenActingPlayer();
		given(confusionSkill.getName()).willReturn("Bone Head");

		ReportConfusionRoll report = new ReportConfusionRoll("p1", true, 4, 2, false, confusionSkill);
		List<Run> runs = render(new ConfusionRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " is able to act normally.".equals(r.text)));
		Run needed = runs.stream().filter(r -> r.textStyle == TextStyle.NEEDED_ROLL).findFirst().orElseThrow();
		assertEquals("Succeeded on a roll of 2+.", needed.text);
	}

	@Test
	public void failedBoneHeadUsesDistractedMessage() {
		givenActingPlayer();
		given(confusionSkill.getName()).willReturn("Bone Head");
		given(confusionSkill.getConfusionMessage()).willReturn("is distracted");

		ReportConfusionRoll report = new ReportConfusionRoll("p1", false, 1, 2, false, confusionSkill);
		List<Run> runs = render(new ConfusionRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " is distracted.".equals(r.text)));
	}

	@Test
	public void failedWildAnimalUsesRoarsInRageMessage() {
		givenActingPlayer();
		given(confusionSkill.getName()).willReturn("Wild Animal");
		given(confusionSkill.getConfusionMessage()).willReturn("roars in rage");

		ReportConfusionRoll report = new ReportConfusionRoll("p1", false, 1, 2, false, confusionSkill);
		List<Run> runs = render(new ConfusionRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " roars in rage.".equals(r.text)));
	}

	@Test
	public void failedTakeRootUsesTakesRootMessage() {
		givenActingPlayer();
		given(confusionSkill.getName()).willReturn("Take Root");
		given(confusionSkill.getConfusionMessage()).willReturn("takes root");

		ReportConfusionRoll report = new ReportConfusionRoll("p1", false, 1, 2, false, confusionSkill);
		List<Run> runs = render(new ConfusionRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " takes root.".equals(r.text)));
	}

	@Test
	public void failedAnimalSavageryUsesLashOutMessage() {
		givenActingPlayer();
		given(confusionSkill.getName()).willReturn("Animal Savagery");
		given(confusionSkill.getConfusionMessage()).willReturn("tries to lash out against a team mate");

		ReportConfusionRoll report = new ReportConfusionRoll("p1", false, 1, 2, false, confusionSkill);
		List<Run> runs = render(new ConfusionRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " tries to lash out against a team mate.".equals(r.text)));
	}

	@Test
	public void failedUnchannelledFuryUsesRoarsInRageMessage() {
		givenActingPlayer();
		given(confusionSkill.getName()).willReturn("Unchannelled Fury");
		given(confusionSkill.getConfusionMessage()).willReturn("roars in rage");

		ReportConfusionRoll report = new ReportConfusionRoll("p1", false, 1, 2, false, confusionSkill);
		List<Run> runs = render(new ConfusionRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " roars in rage.".equals(r.text)));
	}

	@Test
	public void reallyStupidAppendsPropertyNote() {
		givenActingPlayer();
		given(confusionSkill.getName()).willReturn("Really Stupid");
		given(confusionSkill.getConfusionMessage()).willReturn("is distracted");
		given(confusionSkill.hasSkillProperty(NamedProperties.needsToRollHighToAvoidConfusion)).willReturn(true);

		ReportConfusionRoll report = new ReportConfusionRoll("p1", false, 1, 3, false, confusionSkill);
		List<Run> runs = render(new ConfusionRollMessage(), report);

		Run needed = runs.stream().filter(r -> r.textStyle == TextStyle.NEEDED_ROLL).findFirst().orElseThrow();
		assertTrue(needed.text.contains("Really Stupid player without assistance"));
	}

	@Test
	public void rerolledSkipsNeededRoll() {
		givenActingPlayer();
		given(confusionSkill.getName()).willReturn("Bone Head");
		given(confusionSkill.getConfusionMessage()).willReturn("is distracted");

		ReportConfusionRoll report = new ReportConfusionRoll("p1", false, 1, 2, true, confusionSkill);
		List<Run> runs = render(new ConfusionRollMessage(), report);

		assertTrue(runs.stream().noneMatch(r -> r.textStyle == TextStyle.NEEDED_ROLL));
	}

	@Test
	public void reportIdIsConfusionRoll() {
		assertEquals("confusionRoll", new ConfusionRollMessage().getKey());
	}
}
