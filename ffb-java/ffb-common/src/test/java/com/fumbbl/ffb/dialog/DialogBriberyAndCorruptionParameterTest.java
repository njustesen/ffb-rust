package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonValue;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_bribery_and_corruption_parameter.rs for
 * {@link DialogBriberyAndCorruptionParameter}.
 */
public class DialogBriberyAndCorruptionParameterTest {

	@Test
	public void serdeRoundTrip() {
		DialogBriberyAndCorruptionParameter p = new DialogBriberyAndCorruptionParameter("teamX");
		JsonValue json = p.toJsonValue();
		DialogBriberyAndCorruptionParameter back = (DialogBriberyAndCorruptionParameter) new DialogBriberyAndCorruptionParameter()
			.initFrom(NetCommandTestUtil.applicationSource(), json);
		assertEquals("teamX", back.getTeamId());
	}

}
