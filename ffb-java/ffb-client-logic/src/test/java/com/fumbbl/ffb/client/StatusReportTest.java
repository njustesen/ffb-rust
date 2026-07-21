package com.fumbbl.ffb.client;

import com.fumbbl.ffb.FantasyFootballConstants;
import com.fumbbl.ffb.client.ui.LogComponent;
import com.fumbbl.ffb.modifiers.RollModifier;
import com.fumbbl.ffb.net.ServerStatus;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Answers;
import org.mockito.ArgumentCaptor;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import java.util.ArrayList;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;
import static org.mockito.Mockito.atLeast;
import static org.mockito.Mockito.verify;

/**
 * 1:1 translation of `status_report.rs`'s `#[cfg(test)] mod tests`. The Rust `StatusReport`
 * is a headless struct that records emitted runs directly into `rendered_runs`; the real Java
 * `StatusReport` instead calls `getClient().getUserInterface().getLog().append(...)`, so this
 * mirrors the idiom already established in `report/ReportMessageTestBase.java`: a deep-stub
 * mock of `FantasyFootballClient` plus an `ArgumentCaptor` on `LogComponent.append(...)` stands
 * in for the Rust `rendered_runs` list.
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class StatusReportTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	private FantasyFootballClient client;

	private StatusReport statusReport;
	private LogComponent log;

	private static final class Run {
		final ParagraphStyle paragraphStyle;
		final TextStyle textStyle;
		final String text;

		Run(ParagraphStyle paragraphStyle, TextStyle textStyle, String text) {
			this.paragraphStyle = paragraphStyle;
			this.textStyle = textStyle;
			this.text = text;
		}
	}

	@BeforeEach
	void setUp() {
		statusReport = new StatusReport(client);
		log = client.getUserInterface().getLog();
	}

	private List<Run> capturedRuns() {
		ArgumentCaptor<ParagraphStyle> paragraphCaptor = ArgumentCaptor.forClass(ParagraphStyle.class);
		ArgumentCaptor<TextStyle> textStyleCaptor = ArgumentCaptor.forClass(TextStyle.class);
		ArgumentCaptor<String> textCaptor = ArgumentCaptor.forClass(String.class);
		verify(log, atLeast(0)).append(paragraphCaptor.capture(), textStyleCaptor.capture(), textCaptor.capture());
		List<ParagraphStyle> paragraphs = paragraphCaptor.getAllValues();
		List<TextStyle> styles = textStyleCaptor.getAllValues();
		List<String> texts = textCaptor.getAllValues();
		List<Run> runs = new ArrayList<>();
		for (int i = 0; i < texts.size(); i++) {
			runs.add(new Run(paragraphs.get(i), styles.get(i), texts.get(i)));
		}
		return runs;
	}

	@Test
	void reportVersionEmitsSingleRun() {
		statusReport.reportVersion();
		List<Run> runs = capturedRuns();
		assertEquals(2, runs.size());
		assertEquals("FantasyFootballClient Version " + FantasyFootballConstants.VERSION, runs.get(0).text);
		assertEquals(ParagraphStyle.INDENT_0, runs.get(0).paragraphStyle);
	}

	@Test
	void reportTimeoutUsesSpaceAboveBelowAndBold() {
		statusReport.reportTimeout();
		List<Run> runs = capturedRuns();
		assertEquals(ParagraphStyle.SPACE_ABOVE_BELOW, runs.get(0).paragraphStyle);
		assertEquals(TextStyle.BOLD, runs.get(0).textStyle);
	}

	@Test
	void reportGameNameSkipsWhenEmpty() {
		statusReport.reportGameName("");
		assertTrue(capturedRuns().isEmpty());
	}

	@Test
	void formatRollModifiersJoinsWithMinus() {
		RollModifier<?> tackleZone = mockModifier("TackleZone");
		RollModifier<?> blizzard = mockModifier("Blizzard");
		assertEquals(" - TackleZone - Blizzard",
			statusReport.formatRollModifiers(new RollModifier<?>[]{tackleZone, blizzard}));
	}

	@Test
	void formatRollModifiersEmpty() {
		assertEquals("", statusReport.formatRollModifiers(new RollModifier<?>[0]));
	}

	@Test
	void printlnEmitsRunAndTerminator() {
		statusReport.println();
		List<Run> runs = capturedRuns();
		assertEquals(2, runs.size());
		assertNull(runs.get(0).text);
		assertNull(runs.get(1).text);
	}

	@Test
	void reportStatusBracketsMessageWithBlankLines() {
		statusReport.reportStatus(ServerStatus.ERROR_UNKNOWN_COACH);
		List<Run> runs = capturedRuns();
		// blank println (2 runs) + message run/terminator (2 runs) + blank println (2 runs).
		assertEquals(6, runs.size());
		assertEquals("Unknown Coach!", runs.get(2).text);
	}

	@SuppressWarnings({"rawtypes", "unchecked"})
	private RollModifier<?> mockModifier(String name) {
		RollModifier modifier = org.mockito.Mockito.mock(RollModifier.class);
		given(modifier.getModifier()).willReturn(1);
		given(modifier.isModifierIncluded()).willReturn(true);
		given(modifier.getName()).willReturn(name);
		return modifier;
	}
}
