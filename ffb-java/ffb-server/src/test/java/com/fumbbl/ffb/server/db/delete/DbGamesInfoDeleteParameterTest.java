package com.fumbbl.ffb.server.db.delete;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class DbGamesInfoDeleteParameterTest {

	@Test
	void initialUpdatedRows() {
		DbGamesInfoDeleteParameter p = new DbGamesInfoDeleteParameter(42);
		assertEquals(0, p.getUpdatedRows());
	}
}
