package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.SkillUse;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.report.ReportSkillUse;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class SkillUseMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Mock
	private Skill blockSkill;

	@Mock
	private Skill dodgeSkill;

	private void givenPlayer(String id, String name) {
		given(game.getPlayerById(id)).willReturn(player);
		given(player.getName()).willReturn(name);
		given(player.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
	}

	@Test
	public void usedSkillWithPlayerRendersUses() {
		givenPlayer("p1", "Griff");
		given(blockSkill.getName()).willReturn("Block");

		ReportSkillUse report = new ReportSkillUse("p1", blockSkill, true, SkillUse.BRING_DOWN_OPPONENT);
		List<Run> runs = render(new SkillUseMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Griff".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.startsWith(" uses Block")));
	}

	@Test
	public void unusedSkillWithPlayerRendersDoesNotUse() {
		givenPlayer("p1", "Griff");
		given(blockSkill.getName()).willReturn("Block");

		ReportSkillUse report = new ReportSkillUse("p1", blockSkill, false, SkillUse.WOULD_NOT_HELP);
		List<Run> runs = render(new SkillUseMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.startsWith(" does not use Block")));
	}

	@Test
	public void noPlayerFoundRendersSkillNameOnly() {
		given(game.getPlayerById("unknown")).willReturn(null);
		given(dodgeSkill.getName()).willReturn("Dodge");

		ReportSkillUse report = new ReportSkillUse("unknown", dodgeSkill, true, SkillUse.AVOID_PUSH);
		List<Run> runs = render(new SkillUseMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.startsWith("Dodge used")));
	}

	@Test
	public void noPlayerIdRendersSkillNameOnly() {
		given(game.getPlayerById(null)).willReturn(null);
		given(dodgeSkill.getName()).willReturn("Dodge");

		ReportSkillUse report = new ReportSkillUse(null, dodgeSkill, false, SkillUse.WOULD_NOT_HELP);
		List<Run> runs = render(new SkillUseMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> r.text != null && r.text.startsWith("Dodge is not used")));
	}

	@Test
	public void reportIdIsSkillUse() {
		assertEquals("skillUse", new SkillUseMessage().getKey());
	}
}
