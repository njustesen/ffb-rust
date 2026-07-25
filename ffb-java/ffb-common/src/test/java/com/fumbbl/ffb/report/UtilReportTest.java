package com.fumbbl.ffb.report;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertThrows;

/**
 * Mirror of ffb-rust crates/ffb-model/src/report/util_report.rs tests.
 * Uses a concrete ReportConfusionRoll (getId() == CONFUSION_ROLL) in place of the Rust test-local
 * report; Rust "panics" map to Java exceptions.
 */
public class UtilReportTest {

	private ReportConfusionRoll report() {
		return new ReportConfusionRoll("p1", true, 4, 2, false, null);
	}

	// rust: matching_id_does_not_panic
	@Test
	public void matchingIdDoesNotPanic() {
		assertDoesNotThrow(() -> UtilReport.validateReportId(report(), ReportId.CONFUSION_ROLL));
	}

	// rust: null_report_panics
	@Test
	public void nullReportPanics() {
		assertThrows(RuntimeException.class, () -> UtilReport.validateReportId(null, ReportId.CONFUSION_ROLL));
	}

	// rust: mismatched_id_panics
	@Test
	public void mismatchedIdPanics() {
		assertThrows(RuntimeException.class, () -> UtilReport.validateReportId(report(), ReportId.DODGE_ROLL));
	}
}
