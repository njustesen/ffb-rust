package com.fumbbl.ffb.server.db.delete;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class DbPlayerMarkersDeleteParameterTest {

	@Test
	void getUpdatedRowsInitial() {
		DbPlayerMarkersDeleteParameter p = new DbPlayerMarkersDeleteParameter("team1");
		assertEquals(0, p.getUpdatedRows());
	}
}
