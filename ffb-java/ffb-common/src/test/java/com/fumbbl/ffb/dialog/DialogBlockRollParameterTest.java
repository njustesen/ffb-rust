package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonValue;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_block_roll_parameter.rs for
 * {@link DialogBlockRollParameter}.
 */
public class DialogBlockRollParameterTest {

	@Test
	public void serdeRoundTrip() {
		DialogBlockRollParameter p = new DialogBlockRollParameter("teamA", 2, new int[] { 3, 5 }, true, false);
		JsonValue json = p.toJsonValue();
		DialogBlockRollParameter back = (DialogBlockRollParameter) new DialogBlockRollParameter()
			.initFrom(NetCommandTestUtil.applicationSource(), json);
		assertEquals("teamA", back.getChoosingTeamId());
		assertEquals(2, back.getNrOfDice());
		assertArrayEquals(new int[] { 3, 5 }, back.getBlockRoll());
		assertTrue(back.hasTeamReRollOption());
		assertFalse(back.hasProReRollOption());
	}

}
