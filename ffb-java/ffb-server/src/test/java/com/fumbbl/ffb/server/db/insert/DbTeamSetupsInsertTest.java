package com.fumbbl.ffb.server.db.insert;

import com.fumbbl.ffb.server.FantasyFootballServer;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.ArgumentCaptor;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.sql.Connection;
import java.sql.PreparedStatement;
import java.sql.SQLException;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

/**
 * Mirrors the Rust {@code db_team_setups_insert} test.
 */
@ExtendWith(MockitoExtension.class)
class DbTeamSetupsInsertTest {

	@Mock
	private FantasyFootballServer server;

	@Mock
	private Connection connection;

	@Mock
	private PreparedStatement statement;

	@Test
	void sqlHas35Placeholders() throws SQLException {
		ArgumentCaptor<String> sqlCaptor = ArgumentCaptor.forClass(String.class);
		when(connection.prepareStatement(anyString())).thenReturn(statement);
		new DbTeamSetupsInsert(server).prepare(connection);
		verify(connection).prepareStatement(sqlCaptor.capture());
		long placeholders = sqlCaptor.getValue().chars().filter(c -> c == '?').count();
		assertEquals(35, placeholders);
	}
}
