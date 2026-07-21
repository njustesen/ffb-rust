package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonValue;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_join_parameter.rs for
 * {@link DialogJoinParameter}.
 */
public class DialogJoinParameterTest {

	@Test
	public void serdeRoundTrip() {
		JsonValue json = new DialogJoinParameter().toJsonValue();
		DialogJoinParameter back = (DialogJoinParameter) new DialogJoinParameter()
			.initFrom(NetCommandTestUtil.applicationSource(), json);
		assertEquals(DialogId.JOIN, back.getId());
	}

}
