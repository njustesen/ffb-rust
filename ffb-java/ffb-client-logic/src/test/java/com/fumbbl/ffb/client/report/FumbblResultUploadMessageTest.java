package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.client.ParagraphStyle;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.report.ReportFumbblResultUpload;
import org.junit.jupiter.api.Test;

import java.util.ArrayList;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class FumbblResultUploadMessageTest extends ReportMessageTestBase {

	private List<String> texts(List<Run> runs) {
		List<String> texts = new ArrayList<>();
		for (Run run : runs) {
			if (run.text != null) {
				texts.add(run.text);
			}
		}
		return texts;
	}

	@Test
	public void successfulUploadReportsOk() {
		ReportFumbblResultUpload report = new ReportFumbblResultUpload(true, "Upload complete");
		List<Run> runs = render(new FumbblResultUploadMessage(), report);

		assertEquals("Fumbbl Result Upload ok", runs.get(0).text);
		assertEquals(ParagraphStyle.SPACE_ABOVE_BELOW, runs.get(0).paragraphStyle);
		assertEquals(TextStyle.BOLD, runs.get(0).textStyle);
	}

	@Test
	public void failedUploadReportsFailedAndStatusLine() {
		ReportFumbblResultUpload report = new ReportFumbblResultUpload(false, "Connection error");
		List<Run> runs = render(new FumbblResultUploadMessage(), report);

		List<String> texts = texts(runs);
		assertTrue(texts.contains("Fumbbl Result Upload failed"));
		assertTrue(texts.contains("Connection error"));
	}

	@Test
	public void uploadStatusUsesIndentPlusOneParagraphStyle() {
		ReportFumbblResultUpload report = new ReportFumbblResultUpload(true, "OK");
		List<Run> runs = render(new FumbblResultUploadMessage(), report);

		// second println run is the upload status line
		assertEquals(ParagraphStyle.INDENT_1, runs.get(2).paragraphStyle);
		assertEquals("OK", runs.get(2).text);
	}
}
