package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonValue;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_pick_up_choice_parameter.rs for
 * {@link DialogPickUpChoiceParameter}.
 */
public class DialogPickUpChoiceParameterTest {

	@Test
	public void serdeRoundTrip() {
		JsonValue json = new DialogPickUpChoiceParameter().toJsonValue();
		DialogPickUpChoiceParameter back = (DialogPickUpChoiceParameter) new DialogPickUpChoiceParameter()
			.initFrom(NetCommandTestUtil.applicationSource(), json);
		assertEquals(DialogId.PICK_UP_CHOICE, back.getId());
	}

}
