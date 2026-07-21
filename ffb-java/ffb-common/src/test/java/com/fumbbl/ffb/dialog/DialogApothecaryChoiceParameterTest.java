package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonValue;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_apothecary_choice_parameter.rs for
 * {@link DialogApothecaryChoiceParameter}.
 */
public class DialogApothecaryChoiceParameterTest {

	@Test
	public void serdeRoundTrip() {
		DialogApothecaryChoiceParameter p = new DialogApothecaryChoiceParameter("p42", null, null, null, null);
		JsonValue json = p.toJsonValue();
		DialogApothecaryChoiceParameter back = (DialogApothecaryChoiceParameter) new DialogApothecaryChoiceParameter()
			.initFrom(NetCommandTestUtil.applicationSource(), json);
		assertEquals("p42", back.getPlayerId());
	}

}
