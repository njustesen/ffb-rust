package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.mixed.ReportDoubleHiredStaff;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class DoubleHiredStaffMessageTest extends ReportMessageTestBase {

	@Test
	public void rendersStaffNameInBold() {
		ReportDoubleHiredStaff report = new ReportDoubleHiredStaff("Mr. Wibble");
		List<Run> runs = render(new DoubleHiredStaffMessage(), report);

		assertEquals("Inamous Coaching Staff Mr. Wibble takes money from both teams and plays for neither.",
			runs.get(0).text);
		assertEquals(TextStyle.BOLD, runs.get(0).textStyle);
	}

	@Test
	public void rendersAtCurrentIndent() {
		statusReport.setIndent(2);
		ReportDoubleHiredStaff report = new ReportDoubleHiredStaff("Coach");
		List<Run> runs = render(new DoubleHiredStaffMessage(), report);

		assertTrue(runs.get(0).text.contains("Coach"));
	}

	@Test
	public void rendersNullStaffNameGracefully() {
		ReportDoubleHiredStaff report = new ReportDoubleHiredStaff(null);
		List<Run> runs = render(new DoubleHiredStaffMessage(), report);

		assertEquals("Inamous Coaching Staff null takes money from both teams and plays for neither.",
			runs.get(0).text);
	}
}
