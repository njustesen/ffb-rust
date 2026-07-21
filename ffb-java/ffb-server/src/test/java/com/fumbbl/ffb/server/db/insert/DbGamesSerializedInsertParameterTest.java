package com.fumbbl.ffb.server.db.insert;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the Rust {@code db_games_serialized_insert_parameter} test. The Java
 * constructor takes a GameState (Rust takes id + bytes); a null GameState leaves
 * the inherited {@code updatedRows} counter at its 0 default.
 */
class DbGamesSerializedInsertParameterTest {

	@Test
	void initialUpdatedRows() {
		DbGamesSerializedInsertParameter p = new DbGamesSerializedInsertParameter(null);
		assertEquals(0, p.getUpdatedRows());
	}
}
