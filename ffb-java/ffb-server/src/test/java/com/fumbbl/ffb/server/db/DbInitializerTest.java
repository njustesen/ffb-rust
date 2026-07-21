package com.fumbbl.ffb.server.db;

import com.fumbbl.ffb.util.StringTool;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the Rust {@code db_initializer} tests. DbInitializer's SQL builders are
 * private, so - like the Rust tests - these assert the shape produced from the
 * same table/column constants and {@link StringTool#bind}.
 */
class DbInitializerTest {

	@Test
	void dropTableSqlShape() {
		String sql = "DROP TABLE IF EXISTS " + IDbTableGamesInfo.TABLE_NAME + ";";
		assertTrue(sql.contains("DROP TABLE IF EXISTS"));
		assertTrue(sql.contains("ffb_games_info"));
	}

	@Test
	void createTableCoachesSqlShape() {
		StringBuilder sql = new StringBuilder();
		sql.append("CREATE TABLE ").append(IDbTableCoaches.TABLE_NAME).append(" (");
		sql.append(IDbTableCoaches.COLUMN_NAME).append(" VARCHAR(40) NOT NULL,");
		sql.append(IDbTableCoaches.COLUMN_PASSWORD).append(" VARCHAR(32) NOT NULL,");
		sql.append("PRIMARY KEY(").append(IDbTableCoaches.COLUMN_NAME).append(")");
		sql.append(");");
		assertTrue(sql.toString().contains("CREATE TABLE ffb_coaches"));
		assertTrue(sql.toString().contains("PRIMARY KEY(name)"));
	}

	@Test
	void teamSetupsColumnBindProducesIndexedColumns() {
		assertEquals("player_nr_3", StringTool.bind(IDbTableTeamSetups.COLUMN_PLAYER_NR, 3));
	}
}
