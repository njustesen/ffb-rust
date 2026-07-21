package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.report.ReportBlockRoll;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertTrue;

class BlockRollMessageTest extends ReportMessageTestBase {

	@Test
	public void emptyRollRendersNothing() {
		ReportBlockRoll report = new ReportBlockRoll("home", new int[]{});
		List<Run> runs = render(new BlockRollMessage(), report);

		assertTrue(runs.isEmpty());
	}
}
