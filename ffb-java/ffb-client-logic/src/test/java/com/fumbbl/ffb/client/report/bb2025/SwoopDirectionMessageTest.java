package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.Direction;
import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.report.bb2025.ReportSwoopDirection;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;
import java.util.stream.Collectors;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class SwoopDirectionMessageTest extends ReportMessageTestBase {

	@Mock
	@SuppressWarnings("rawtypes")
	private Player player;

	private void givenIdentityMapping(Direction direction) {
		given(client.getUserInterface().getPitchDimensionProvider().mapToLocal(direction)).willReturn(direction);
	}

	@Test
	public void inBoundsSwoopHasNoDativeClause() {
		givenIdentityMapping(Direction.EAST);
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getPlayerGender()).willReturn(PlayerGender.MALE);

		ReportSwoopDirection report = new ReportSwoopDirection(Direction.EAST, 3, "p1", false);
		List<Run> runs = render(new SwoopDirectionMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(Collectors.toList());
		assertEquals(true, texts.stream().anyMatch(t -> "Swoop Direction Roll [ 3 ] East".equals(t)));
		assertEquals(true, texts.stream().anyMatch(t -> " swoops East.".equals(t)));
	}

	@Test
	public void outOfBoundsSwoopIncludesDative() {
		givenIdentityMapping(Direction.NORTH);
		given(game.getPlayerById("p1")).willReturn(player);
		given(player.getPlayerGender()).willReturn(PlayerGender.MALE);

		ReportSwoopDirection report = new ReportSwoopDirection(Direction.NORTH, 6, "p1", true);
		List<Run> runs = render(new SwoopDirectionMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(Collectors.toList());
		assertEquals(true, texts.stream().anyMatch(t -> " swoops North which takes him out of bounds.".equals(t)));
	}

	@Test
	public void dativeMatchesGender() {
		assertEquals("him", PlayerGender.MALE.getDative());
		assertEquals("her", PlayerGender.FEMALE.getDative());
		assertEquals("them", PlayerGender.NONBINARY.getDative());
		assertEquals("it", PlayerGender.NEUTRAL.getDative());
	}

	@Test
	public void reportIdIsSwoopDirectionRoll() {
		assertEquals(ReportId.SWOOP_DIRECTION_ROLL.getKey(), new SwoopDirectionMessage().getKey());
	}
}
