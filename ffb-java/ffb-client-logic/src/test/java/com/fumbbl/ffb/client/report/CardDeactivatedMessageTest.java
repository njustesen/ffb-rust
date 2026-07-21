package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.client.ParagraphStyle;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.inducement.Card;
import com.fumbbl.ffb.report.ReportCardDeactivated;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class CardDeactivatedMessageTest extends ReportMessageTestBase {

	@Mock
	private Card card;

	@Test
	public void rendersCardNameAndEndedText() {
		given(card.getName()).willReturn("CUSTARD_PIE");

		ReportCardDeactivated report = new ReportCardDeactivated(card);
		List<Run> runs = render(new CardDeactivatedMessage(), report);

		assertEquals("Card CUSTARD_PIE effect ended.", runs.get(0).text);
		assertEquals(TextStyle.BOLD, runs.get(0).textStyle);
	}

	@Test
	public void rendersDifferentCardName() {
		given(card.getName()).willReturn("ILLEGAL_PROCEDURE");

		ReportCardDeactivated report = new ReportCardDeactivated(card);
		List<Run> runs = render(new CardDeactivatedMessage(), report);

		assertEquals("Card ILLEGAL_PROCEDURE effect ended.", runs.get(0).text);
	}

	@Test
	public void respectsCurrentIndent() {
		statusReport.setIndent(2);
		given(card.getName()).willReturn("BRIBE");

		ReportCardDeactivated report = new ReportCardDeactivated(card);
		List<Run> runs = render(new CardDeactivatedMessage(), report);

		assertEquals(ParagraphStyle.INDENT_2, runs.get(0).paragraphStyle);
	}

	@Test
	public void reportIdIsCardDeactivated() {
		assertEquals("cardDeactivated", new CardDeactivatedMessage().getKey());
	}
}
