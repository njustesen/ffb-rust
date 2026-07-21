package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonValue;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_select_gaze_target_parameter.rs for
 * {@link DialogSelectGazeTargetParameter}.
 */
public class DialogSelectGazeTargetParameterTest {

	@Test
	public void serdeRoundTrip() {
		JsonValue json = new DialogSelectGazeTargetParameter().toJsonValue();
		DialogId id = new DialogSelectGazeTargetParameter()
			.initFrom(NetCommandTestUtil.applicationSource(), json).getId();
		assertEquals(DialogId.SELECT_GAZE_TARGET, id);
	}

}
