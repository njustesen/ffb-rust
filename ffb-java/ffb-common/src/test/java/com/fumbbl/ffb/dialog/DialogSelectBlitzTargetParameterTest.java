package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonValue;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_select_blitz_target_parameter.rs for
 * {@link DialogSelectBlitzTargetParameter}.
 */
public class DialogSelectBlitzTargetParameterTest {

	@Test
	public void serdeRoundTrip() {
		JsonValue json = new DialogSelectBlitzTargetParameter().toJsonValue();
		DialogId id = new DialogSelectBlitzTargetParameter()
			.initFrom(NetCommandTestUtil.applicationSource(), json).getId();
		assertEquals(DialogId.SELECT_BLITZ_TARGET, id);
	}

}
