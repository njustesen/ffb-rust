package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonValue;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_coin_choice_parameter.rs for
 * {@link DialogCoinChoiceParameter}.
 */
public class DialogCoinChoiceParameterTest {

	@Test
	public void serdeRoundTrip() {
		JsonValue json = new DialogCoinChoiceParameter().toJsonValue();
		DialogCoinChoiceParameter back = (DialogCoinChoiceParameter) new DialogCoinChoiceParameter()
			.initFrom(NetCommandTestUtil.applicationSource(), json);
		assertEquals(DialogId.COIN_CHOICE, back.getId());
	}

}
