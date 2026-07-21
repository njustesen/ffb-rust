package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonValue;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_pass_block_parameter.rs for
 * {@link DialogPassBlockParameter}.
 */
public class DialogPassBlockParameterTest {

	@Test
	public void serdeRoundTrip() {
		JsonValue json = new DialogPassBlockParameter().toJsonValue();
		DialogPassBlockParameter back = (DialogPassBlockParameter) new DialogPassBlockParameter()
			.initFrom(NetCommandTestUtil.applicationSource(), json);
		assertEquals(DialogId.PASS_BLOCK, back.getId());
	}

}
