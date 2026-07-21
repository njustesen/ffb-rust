package com.fumbbl.ffb.server.db.update;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;

class DbGamesInfoUpdateParameterTest {

	/**
	 * Mirrors the Rust {@code default_fields} test. The Java constructor derives every field
	 * from a GameState; with a null GameState it leaves all object fields null and all
	 * primitive fields at their defaults.
	 * <p>
	 * Divergence from Rust: Rust's {@code DbGamesInfoUpdateParameter::new(id)} initialises the
	 * string columns (e.g. coach_home) to "" whereas Java leaves them null. The remaining
	 * defaults (timestamps null, half/turn 0, flags false) agree.
	 */
	@Test
	void defaultFields() {
		DbGamesInfoUpdateParameter p = new DbGamesInfoUpdateParameter(null);
		assertNull(p.getScheduled());
		assertNull(p.getStarted());
		assertNull(p.getFinished());
		assertNull(p.getCoachHome());
		assertEquals(0, p.getHalf());
		assertEquals(0, p.getTurn());
		assertFalse(p.isHomePlaying());
		assertFalse(p.isTesting());
		assertFalse(p.isAdminMode());
	}
}
