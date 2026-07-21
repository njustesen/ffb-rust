package com.fumbbl.ffb.client.report.bb2016;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.report.ReportBloodLustRoll;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

class BloodLustRollMessageTest extends ReportMessageTestBase {

	@Test
	public void getKeyIsBloodLustRoll() {
		assertEquals("bloodLustRoll", new BloodLustRollMessage().getKey());
	}

	@Test
	public void successfulRollResistsBloodLust() {
		ReportBloodLustRoll report = new ReportBloodLustRoll("p1", true, 4, 2, false, null);
		List<Run> runs = render(new BloodLustRollMessage(), report);

		assertEquals(" resists the Blood Lust.", runs.get(3).text);
		assertTrue(runs.stream().anyMatch(r -> "Succeeded on a roll of 2+".equals(r.text)));
	}

	@Test
	public void failedRollGivesIn() {
		ReportBloodLustRoll report = new ReportBloodLustRoll("p1", false, 1, 2, false, null);
		List<Run> runs = render(new BloodLustRollMessage(), report);

		assertEquals(" gives in to the Blood Lust.", runs.get(3).text);
	}

	@Test
	public void reRolledHasNoNeededRollLine() {
		ReportBloodLustRoll report = new ReportBloodLustRoll("p1", false, 1, 2, true, null);
		List<Run> runs = render(new BloodLustRollMessage(), report);

		assertFalse(runs.stream().anyMatch(r -> r.textStyle == TextStyle.NEEDED_ROLL));
	}
}
