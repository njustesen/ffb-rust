package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.CardEffect;
import com.fumbbl.ffb.inducement.Card;
import com.fumbbl.ffb.inducement.CardReport;
import com.fumbbl.ffb.report.ReportCardEffectRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;
import java.util.Optional;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyInt;
import static org.mockito.BDDMockito.given;

class CardEffectRollMessageTest extends ReportMessageTestBase {

	@Mock
	private Card card;

	@Test
	public void rendersNothingWhenNoCardEffect() {
		given(card.cardReport(any(), anyInt())).willReturn(Optional.empty());

		ReportCardEffectRoll report = new ReportCardEffectRoll(card, 3);
		List<Run> runs = render(new CardEffectRollMessage(), report);

		assertTrue(runs.isEmpty());
	}

	@Test
	public void rendersNothingWhenCardEffectPresent() {
		given(card.cardReport(any(), anyInt())).willReturn(Optional.empty());

		ReportCardEffectRoll report = new ReportCardEffectRoll(card, 5);
		report.setCardEffect(CardEffect.MAD_CAP_MUSHROOM_POTION);
		List<Run> runs = render(new CardEffectRollMessage(), report);

		assertTrue(runs.isEmpty());
	}

	@Test
	public void reportIdIsCardEffectRoll() {
		assertEquals("cardEffectRoll", new CardEffectRollMessage().getKey());
	}

	@Test
	public void witchsBrewNoEffectRendersSnakeOil() {
		given(card.cardReport(any(), anyInt())).willReturn(
			Optional.of(new CardReport("Witch Brew Roll [ 2 ]", "Snake Oil! Bad taste, but no effect.")));

		ReportCardEffectRoll report = new ReportCardEffectRoll(card, 2);
		List<Run> runs = render(new CardEffectRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Witch Brew Roll [ 2 ]".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> "Snake Oil! Bad taste, but no effect.".equals(r.text)));
	}

	@Test
	public void witchsBrewSedativeRendersEffectText() {
		given(card.cardReport(any(), anyInt())).willReturn(
			Optional.of(new CardReport("Witch Brew Roll [ 3 ]",
				"Sedative! The player gains the Really Stupid skill until the drive ends.")));

		ReportCardEffectRoll report = new ReportCardEffectRoll(card, 3);
		report.setCardEffect(CardEffect.SEDATIVE);
		List<Run> runs = render(new CardEffectRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Witch Brew Roll [ 3 ]".equals(r.text)));
		assertTrue(runs.stream().anyMatch(
			r -> "Sedative! The player gains the Really Stupid skill until the drive ends.".equals(r.text)));
	}

	@Test
	public void witchsBrewMadCapMushroomPotionRendersEffectText() {
		given(card.cardReport(any(), anyInt())).willReturn(
			Optional.of(new CardReport("Witch Brew Roll [ 1 ]",
				"Mad Cap Mushroom potion! The player gains the Jump Up and No Hands skills until the drive ends.")));

		ReportCardEffectRoll report = new ReportCardEffectRoll(card, 1);
		report.setCardEffect(CardEffect.MAD_CAP_MUSHROOM_POTION);
		List<Run> runs = render(new CardEffectRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Witch Brew Roll [ 1 ]".equals(r.text)));
		assertTrue(runs.stream().anyMatch(
			r -> "Mad Cap Mushroom potion! The player gains the Jump Up and No Hands skills until the drive ends.".equals(r.text)));
	}
}
