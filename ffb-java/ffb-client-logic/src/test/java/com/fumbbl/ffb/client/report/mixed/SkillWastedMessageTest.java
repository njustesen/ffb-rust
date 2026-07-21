package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.report.mixed.ReportSkillWasted;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class SkillWastedMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Mock
	private Skill skill;

	@Test
	public void skillNoneRendersNothing() {
		ReportSkillWasted report = new ReportSkillWasted("p1", null);
		List<Run> runs = render(new SkillWastedMessage(), report);

		assertTrue(runs.isEmpty());
	}

	@Test
	public void skillWithPlayer() {
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getName()).willReturn("Wastey");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
		given(skill.getName()).willReturn("Dodge");

		ReportSkillWasted report = new ReportSkillWasted("p1", skill);
		List<Run> runs = render(new SkillWastedMessage(), report);

		assertEquals("Wastey", runs.get(0).text);
		assertEquals(" wastes Dodge.", runs.get(1).text);
	}

	@Test
	public void skillWithoutPlayer() {
		// without this stub, the deep-stub Game mock would auto-vivify a non-null Player
		// mock for the unstubbed getPlayerById(null) call, breaking the "no player" branch.
		given(game.getPlayerById((String) null)).willReturn(null);
		given(skill.getName()).willReturn("Dodge");

		ReportSkillWasted report = new ReportSkillWasted(null, skill);
		List<Run> runs = render(new SkillWastedMessage(), report);

		assertEquals("Dodge is wasted.", runs.get(0).text);
	}
}
