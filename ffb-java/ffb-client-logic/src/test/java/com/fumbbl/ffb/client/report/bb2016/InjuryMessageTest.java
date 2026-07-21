package com.fumbbl.ffb.client.report.bb2016;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.injury.Block;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.modifiers.ArmorModifier;
import com.fumbbl.ffb.modifiers.InjuryModifier;
import com.fumbbl.ffb.report.bb2016.ReportInjury;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class InjuryMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player defender;

	@Test
	public void getKeyIsInjury() {
		assertEquals("injury", new InjuryMessage().getKey());
	}

	@Test
	public void armorSavedReportsNoBreak() {
		given(game.getPlayerById("defender")).willReturn(defender);
		given(defender.getPlayerGender()).willReturn(PlayerGender.MALE);

		ReportInjury report = new ReportInjury("defender", new Block(), false, new ArmorModifier[]{}, new int[]{3, 3},
			new InjuryModifier[]{}, new int[]{}, new int[]{}, null, new int[]{}, null, null, null, null);
		List<Run> runs = render(new InjuryMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " has been saved by his armour.".equals(r.text)));
	}

	@Test
	public void armorBrokenReportsBreakAndInjury() {
		given(game.getPlayerById("defender")).willReturn(defender);
		given(defender.getPlayerGender()).willReturn(PlayerGender.MALE);

		ReportInjury report = new ReportInjury("defender", new Block(), true, new ArmorModifier[]{}, new int[]{5, 5},
			new InjuryModifier[]{}, new int[]{3, 4}, new int[]{}, null, new int[]{}, null,
			new PlayerState(PlayerState.BADLY_HURT), null, null);
		List<Run> runs = render(new InjuryMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " has been broken.".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> " has been badly hurt.".equals(r.text)));
	}

	@Test
	public void casualtyRollReportsCasualtyAndInjury() {
		given(game.getPlayerById("defender")).willReturn(defender);
		given(defender.getPlayerGender()).willReturn(PlayerGender.MALE);

		ReportInjury report = new ReportInjury("defender", new Block(), true, new ArmorModifier[]{}, new int[]{5, 5},
			new InjuryModifier[]{}, new int[]{3, 4}, new int[]{2, 2}, null, new int[]{}, null,
			new PlayerState(PlayerState.BADLY_HURT), null, null);
		List<Run> runs = render(new InjuryMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> " suffers a casualty.".equals(r.text)));
	}
}
