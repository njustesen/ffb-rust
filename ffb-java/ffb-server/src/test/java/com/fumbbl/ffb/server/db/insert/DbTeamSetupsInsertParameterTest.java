package com.fumbbl.ffb.server.db.insert;

import com.fumbbl.ffb.TeamSetup;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the Rust {@code db_team_setups_insert_parameter} test. The Java
 * constructor takes a {@link TeamSetup}; an empty setup leaves the inherited
 * {@code updatedRows} counter at 0.
 */
class DbTeamSetupsInsertParameterTest {

	@Test
	void getUpdatedRowsInitial() {
		DbTeamSetupsInsertParameter p = new DbTeamSetupsInsertParameter(new TeamSetup());
		assertEquals(0, p.getUpdatedRows());
	}
}
