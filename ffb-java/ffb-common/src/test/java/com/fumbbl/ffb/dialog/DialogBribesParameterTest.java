package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonValue;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_bribes_parameter.rs for
 * {@link DialogBribesParameter}.
 */
public class DialogBribesParameterTest {

	@Test
	public void addPlayerIdAppendsNonemptyStrings() {
		DialogBribesParameter p = new DialogBribesParameter("t", 2);
		p.addPlayerId("p1");
		p.addPlayerId("");
		assertArrayEquals(new String[] { "p1" }, p.getPlayerIds());
	}

	@Test
	public void serdeRoundTrip() {
		DialogBribesParameter p = new DialogBribesParameter("team2", 5);
		JsonValue json = p.toJsonValue();
		DialogBribesParameter back = (DialogBribesParameter) new DialogBribesParameter()
			.initFrom(NetCommandTestUtil.applicationSource(), json);
		assertEquals(5, back.getMaxNrOfBribes());
		assertEquals("team2", back.getTeamId());
	}

}
