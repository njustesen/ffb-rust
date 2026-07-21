package com.fumbbl.ffb.model;

import com.eclipsesource.json.JsonValue;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/roster_position.rs for {@link RosterPosition}.
 */
public class RosterPositionTest {

	@Test
	public void serdeRoundTrip() {
		RosterPosition pos = new RosterPosition("lineman");
		pos.setName("Lineman");
		pos.setMovement(6);
		JsonValue json = pos.toJsonValue();
		RosterPosition back = new RosterPosition().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals("lineman", back.getId());
		assertEquals(6, back.getMovement());
	}

}
