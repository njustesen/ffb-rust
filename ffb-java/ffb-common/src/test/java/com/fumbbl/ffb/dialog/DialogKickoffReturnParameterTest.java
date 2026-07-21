package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonValue;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_kickoff_return_parameter.rs for
 * {@link DialogKickoffReturnParameter}.
 */
public class DialogKickoffReturnParameterTest {

	@Test
	public void serdeRoundTrip() {
		JsonValue json = new DialogKickoffReturnParameter().toJsonValue();
		DialogKickoffReturnParameter back = (DialogKickoffReturnParameter) new DialogKickoffReturnParameter()
			.initFrom(NetCommandTestUtil.applicationSource(), json);
		assertEquals(DialogId.KICKOFF_RETURN, back.getId());
	}

}
