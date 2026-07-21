package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonValue;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_buy_inducements_parameter.rs for
 * {@link DialogBuyInducementsParameter}.
 */
public class DialogBuyInducementsParameterTest {

	@Test
	public void serdeRoundTrip() {
		DialogBuyInducementsParameter p = new DialogBuyInducementsParameter("teamY", 200_000);
		JsonValue json = p.toJsonValue();
		DialogBuyInducementsParameter back = (DialogBuyInducementsParameter) new DialogBuyInducementsParameter()
			.initFrom(NetCommandTestUtil.applicationSource(), json);
		assertEquals("teamY", back.getTeamId());
		assertEquals(200_000, back.getAvailableGold());
	}

}
