package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonValue;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_bloodlust_action_parameter.rs for
 * {@link DialogBloodlustActionParameter}.
 */
public class DialogBloodlustActionParameterTest {

	@Test
	public void serdeRoundTrip() {
		DialogBloodlustActionParameter p = new DialogBloodlustActionParameter(true);
		JsonValue json = p.toJsonValue();
		DialogBloodlustActionParameter back = (DialogBloodlustActionParameter) new DialogBloodlustActionParameter()
			.initFrom(NetCommandTestUtil.applicationSource(), json);
		assertTrue(back.isChangeToMove());
	}

}
