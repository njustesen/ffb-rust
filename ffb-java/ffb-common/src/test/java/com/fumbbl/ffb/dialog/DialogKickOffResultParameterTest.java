package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonValue;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_kick_off_result_parameter.rs for
 * {@link DialogKickOffResultParameter}.
 */
public class DialogKickOffResultParameterTest {

	@Test
	public void serdeRoundTrip() {
		DialogKickOffResultParameter p = new DialogKickOffResultParameter("home");
		JsonValue json = p.toJsonValue();
		DialogKickOffResultParameter back = (DialogKickOffResultParameter) new DialogKickOffResultParameter()
			.initFrom(NetCommandTestUtil.applicationSource(), json);
		assertEquals("home", back.getTeamId());
	}

}
