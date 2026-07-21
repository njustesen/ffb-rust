package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonValue;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_defender_action_parameter.rs for
 * {@link DialogDefenderActionParameter}.
 */
public class DialogDefenderActionParameterTest {

	@Test
	public void serdeRoundTrip() {
		JsonValue json = new DialogDefenderActionParameter().toJsonValue();
		DialogDefenderActionParameter back = (DialogDefenderActionParameter) new DialogDefenderActionParameter()
			.initFrom(NetCommandTestUtil.applicationSource(), json);
		assertEquals(DialogId.DEFENDER_ACTION, back.getId());
	}

}
