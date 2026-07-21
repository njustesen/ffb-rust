package com.fumbbl.ffb.model;

import com.eclipsesource.json.JsonValue;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/roster.rs for {@link Roster}.
 */
public class RosterTest {

	@Test
	public void serdeRoundTrip() {
		Roster r = new Roster();
		r.setId("human");
		r.setReRollCost(50000);
		JsonValue json = r.toJsonValue();
		Roster back = new Roster().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals("human", back.getId());
		assertEquals(50000, back.getReRollCost());
	}

	@Test
	public void hasVampireLordFalseWithoutKeyword() {
		Roster r = new Roster();
		assertFalse(r.hasVampireLord());
	}

	@Test
	public void hasVampireLordTrueWithKeyword() {
		Roster r = new Roster();
		r.getKeywords().add(Keyword.VAMPIRE_LORD);
		assertTrue(r.hasVampireLord());
	}

}
