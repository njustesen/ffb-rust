package com.fumbbl.ffb.server.db.delete;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class DbGamesSerializedDeleteParameterTest {

	@Test
	void initialUpdatedRows() {
		DbGamesSerializedDeleteParameter p = new DbGamesSerializedDeleteParameter(7);
		assertEquals(0, p.getUpdatedRows());
	}
}
