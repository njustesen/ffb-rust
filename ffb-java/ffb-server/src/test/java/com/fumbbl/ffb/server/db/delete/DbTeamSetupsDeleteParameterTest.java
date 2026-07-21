package com.fumbbl.ffb.server.db.delete;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class DbTeamSetupsDeleteParameterTest {

	@Test
	void initialUpdatedRowsIsZero() {
		DbTeamSetupsDeleteParameter p = new DbTeamSetupsDeleteParameter("team1", "setup1");
		assertEquals(0, p.getUpdatedRows());
	}
}
