package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.mixed.ReportProjectileVomit;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class ProjectileVomitMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player thrower;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player defender;

	@Test
	public void successfulVomitPrintsDefender() {
		given(game.getActingPlayer().getPlayer()).willReturn(thrower);
		given(thrower.getName()).willReturn("Joe");
		given(game.getTeamHome().hasPlayer(thrower)).willReturn(true);
		given(game.getPlayerById("p2")).willReturn(defender);
		given(defender.getName()).willReturn("Jane");
		given(game.getTeamHome().hasPlayer(defender)).willReturn(false);

		ReportProjectileVomit report = new ReportProjectileVomit("p1", true, 4, 2, false, "p2");
		List<Run> runs = render(new ProjectileVomitMessage(), report);

		assertEquals("Projectile Vomit Roll [ 4 ]", runs.get(0).text);
		assertEquals(true, runs.stream().anyMatch(r -> "Jane".equals(r.text)));
		Run last = runs.get(runs.size() - 2);
		assertEquals(".", last.text);
	}

	@Test
	public void unsuccessfulVomitUsesGenderSelfMale() {
		given(game.getActingPlayer().getPlayer()).willReturn(thrower);
		given(thrower.getName()).willReturn("Joe");
		given(thrower.getPlayerGender()).willReturn(PlayerGender.MALE);
		given(game.getTeamHome().hasPlayer(thrower)).willReturn(true);

		ReportProjectileVomit report = new ReportProjectileVomit("p1", false, 2, 4, false, null);
		List<Run> runs = render(new ProjectileVomitMessage(), report);

		Run last = runs.get(runs.size() - 2);
		assertEquals(" vomits on himself.", last.text);
	}

	@Test
	public void unsuccessfulVomitUsesGenderSelfFemale() {
		given(game.getActingPlayer().getPlayer()).willReturn(thrower);
		given(thrower.getName()).willReturn("Jane");
		given(thrower.getPlayerGender()).willReturn(PlayerGender.FEMALE);
		given(game.getTeamHome().hasPlayer(thrower)).willReturn(false);

		ReportProjectileVomit report = new ReportProjectileVomit("p2", false, 2, 4, false, null);
		List<Run> runs = render(new ProjectileVomitMessage(), report);

		Run last = runs.get(runs.size() - 2);
		assertEquals(" vomits on herself.", last.text);
	}
}
