package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonValue;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_punt_to_crowd_parameter.rs for
 * {@link DialogPuntToCrowdParameter}.
 */
public class DialogPuntToCrowdParameterTest {

	@Test
	public void serdeRoundTrip() {
		JsonValue json = new DialogPuntToCrowdParameter().toJsonValue();
		DialogPuntToCrowdParameter back = (DialogPuntToCrowdParameter) new DialogPuntToCrowdParameter()
			.initFrom(NetCommandTestUtil.applicationSource(), json);
		assertEquals(DialogId.PUNT_TO_CROWD, back.getId());
	}

}
