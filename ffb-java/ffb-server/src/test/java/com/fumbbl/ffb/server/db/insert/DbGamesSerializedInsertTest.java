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

import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

/**
 * Mirrors the Rust {@code db_games_serialized_insert} test.
 */
@ExtendWith(MockitoExtension.class)
class DbGamesSerializedInsertTest {

	@Mock
	private FantasyFootballServer server;

	@Mock
	private Connection connection;

	@Mock
	private PreparedStatement statement;

	@Test
	void sqlIsInsert() throws SQLException {
		ArgumentCaptor<String> sqlCaptor = ArgumentCaptor.forClass(String.class);
		when(connection.prepareStatement(anyString())).thenReturn(statement);
		new DbGamesSerializedInsert(server).prepare(connection);
		verify(connection).prepareStatement(sqlCaptor.capture());
		assertTrue(sqlCaptor.getValue().trim().toUpperCase().startsWith("INSERT"));
	}
}
