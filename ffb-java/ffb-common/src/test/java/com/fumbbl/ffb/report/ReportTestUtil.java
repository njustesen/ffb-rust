package com.fumbbl.ffb.report;

import com.fumbbl.ffb.factory.IFactorySource;
import com.fumbbl.ffb.net.NetCommandTestUtil;

/**
 * Shared helpers for report serialization tests. Mirrors the wire-contract
 * tests that survive in the ffb-rust crates/ffb-model/src/report tree:
 * {@code serialization_round_trip} (build report -> toJsonValue() -> initFrom()
 * fresh instance -> assert fields equal) and {@code to_json_value_has_report_id}
 * (assert the reportId string in the JSON).
 *
 * <p>Provides an {@link IFactorySource} wired up the way production code does,
 * reusing the game-context source from {@link NetCommandTestUtil}.
 */
public final class ReportTestUtil {

	private ReportTestUtil() {
	}

	/**
	 * A game-context {@link IFactorySource} suitable for {@code initFrom}.
	 */
	public static IFactorySource source() {
		return NetCommandTestUtil.gameSource();
	}
}
