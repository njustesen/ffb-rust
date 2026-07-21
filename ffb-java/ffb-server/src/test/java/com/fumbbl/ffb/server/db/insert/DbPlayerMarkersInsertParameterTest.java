package com.fumbbl.ffb.server.db.insert;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the Rust {@code db_player_markers_insert_parameter} tests.
 *
 * <p>The Rust {@code execute_update_is_jdbc_artifact} test is not ported: the
 * Java {@code executeUpdate(server)} resolves the prepared statement from the
 * server's update factory and needs a live JDBC statement, so it cannot be
 * exercised without a database.
 */
class DbPlayerMarkersInsertParameterTest {

	@Test
	void getUpdatedRowsInitial() {
		DbPlayerMarkersInsertParameter p = new DbPlayerMarkersInsertParameter("t1", "p1", "text");
		assertEquals(0, p.getUpdatedRows());
	}
}
