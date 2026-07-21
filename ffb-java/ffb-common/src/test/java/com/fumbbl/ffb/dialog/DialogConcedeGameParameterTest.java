package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonValue;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_concede_game_parameter.rs for
 * {@link DialogConcedeGameParameter}.
 */
public class DialogConcedeGameParameterTest {

	@Test
	public void serdeRoundTrip() {
		JsonValue json = new DialogConcedeGameParameter().toJsonValue();
		DialogConcedeGameParameter back = (DialogConcedeGameParameter) new DialogConcedeGameParameter()
			.initFrom(NetCommandTestUtil.applicationSource(), json);
		assertEquals(DialogId.CONCEDE_GAME, back.getId());
	}

}
