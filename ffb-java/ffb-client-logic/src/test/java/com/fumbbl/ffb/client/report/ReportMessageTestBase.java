package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.ParagraphStyle;
import com.fumbbl.ffb.client.StatusReport;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.ui.LogComponent;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.report.IReport;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Answers;
import org.mockito.ArgumentCaptor;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import java.util.ArrayList;
import java.util.List;

import static org.mockito.Mockito.atLeast;
import static org.mockito.Mockito.verify;

/**
 * Shared fixture for report message render tests. Mirrors the Rust {@code StatusReport}
 * {@code rendered_runs} inspection: every {@code print}/{@code println} in a report
 * renderer ends up as a {@code LogComponent.append(paragraphStyle, textStyle, text)} call,
 * so we capture those calls in order and expose them as {@link Run}s indexed the same way
 * the Rust tests index {@code rendered_runs}.
 */
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
public abstract class ReportMessageTestBase {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	protected FantasyFootballClient client;

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	protected Game game;

	protected StatusReport statusReport;
	protected LogComponent log;

	@BeforeEach
	public void baseSetUp() {
		statusReport = new StatusReport(client);
		log = client.getUserInterface().getLog();
		org.mockito.BDDMockito.given(client.getGame()).willReturn(game);
	}

	protected static final class Run {
		public final ParagraphStyle paragraphStyle;
		public final TextStyle textStyle;
		public final String text;

		Run(ParagraphStyle paragraphStyle, TextStyle textStyle, String text) {
			this.paragraphStyle = paragraphStyle;
			this.textStyle = textStyle;
			this.text = text;
		}
	}

	protected List<Run> render(ReportMessageBase<? extends IReport> handler, IReport report) {
		handler.setStatusReport(statusReport);
		handler.renderMessage(game, report);
		return capturedRuns();
	}

	protected List<Run> capturedRuns() {
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
}
