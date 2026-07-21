package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.modifiers.GoForItModifier;
import com.fumbbl.ffb.modifiers.RollModifier;
import com.fumbbl.ffb.report.ReportGoForItRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class GoForItRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	@Test
	public void rendersSuccessWithoutReroll() {
		given(game.getActingPlayer().getPlayer()).willReturn(player);
		given(player.getName()).willReturn("Grubber");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportGoForItRoll report = new ReportGoForItRoll("p1", true, 4, 2, false, null);
		List<Run> runs = render(new GoForItRollMessage(), report);

		assertEquals("Rush Roll [ 4 ]", runs.get(0).text);
		assertEquals("Grubber", runs.get(2).text);
		assertEquals(" rushes!", runs.get(3).text);
		assertEquals("Succeeded on a roll of 2+ (Roll > 1).", runs.get(5).text);
		assertEquals(TextStyle.NEEDED_ROLL, runs.get(5).textStyle);
	}

	@Test
	public void rendersFailureWithoutReroll() {
		given(game.getActingPlayer().getPlayer()).willReturn(player);
		given(player.getName()).willReturn("Grubber");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		RollModifier<?>[] modifiers = { new GoForItModifier("TackleZone", 1) };
		ReportGoForItRoll report = new ReportGoForItRoll("p1", false, 1, 3, false, modifiers);
		List<Run> runs = render(new GoForItRollMessage(), report);

		assertEquals(" trips while rushing.", runs.get(3).text);
		// Java's formatRollModifiers includes the modifier magnitude (unlike the Rust helper,
		// which only joins names) since GoForItModifier.isModifierIncluded() is always false.
		assertEquals("Roll a 3+ to succeed (Roll - 1 TackleZone > 2).", runs.get(5).text);
	}

	@Test
	public void skipsNeededRollWhenReRolled() {
		given(game.getActingPlayer().getPlayer()).willReturn(player);
		given(player.getName()).willReturn("Grubber");
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);

		ReportGoForItRoll report = new ReportGoForItRoll("p1", true, 4, 2, true, null);
		List<Run> runs = render(new GoForItRollMessage(), report);

		// roll println (2 runs) + player print (1 run) + " rushes!" println (2 runs); no needed-roll line.
		assertEquals(5, runs.size());
	}
}
