package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonValue;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_game_statistics_parameter.rs for
 * {@link DialogGameStatisticsParameter}.
 */
public class DialogGameStatisticsParameterTest {

	@Test
	public void serdeRoundTrip() {
		JsonValue json = new DialogGameStatisticsParameter().toJsonValue();
		DialogGameStatisticsParameter back = (DialogGameStatisticsParameter) new DialogGameStatisticsParameter()
			.initFrom(NetCommandTestUtil.applicationSource(), json);
		assertEquals(DialogId.GAME_STATISTICS, back.getId());
	}

}
