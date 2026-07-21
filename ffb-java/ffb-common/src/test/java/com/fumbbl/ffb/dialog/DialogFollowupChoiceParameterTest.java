package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonValue;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_followup_choice_parameter.rs for
 * {@link DialogFollowupChoiceParameter}.
 */
public class DialogFollowupChoiceParameterTest {

	@Test
	public void serdeRoundTrip() {
		JsonValue json = new DialogFollowupChoiceParameter().toJsonValue();
		DialogFollowupChoiceParameter back = (DialogFollowupChoiceParameter) new DialogFollowupChoiceParameter()
			.initFrom(NetCommandTestUtil.applicationSource(), json);
		assertEquals(DialogId.FOLLOWUP_CHOICE, back.getId());
	}

}
