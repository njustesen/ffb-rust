package com.fumbbl.ffb.server.db.query;

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
import static org.mockito.ArgumentMatchers.anyInt;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class DbGamesInfoInsertQueryTest {

	@Mock
	private FantasyFootballServer server;

	@Mock
	private Connection connection;

	@Mock
	private PreparedStatement statement;

	@Test
	void sqlHasAllPlaceholders() throws SQLException {
		ArgumentCaptor<String> sqlCaptor = ArgumentCaptor.forClass(String.class);
		// prepare() uses the prepareStatement(sql, RETURN_GENERATED_KEYS) overload
		when(connection.prepareStatement(anyString(), anyInt())).thenReturn(statement);
		new DbGamesInfoInsertQuery(server).prepare(connection);
		verify(connection).prepareStatement(sqlCaptor.capture(), anyInt());
		long placeholders = sqlCaptor.getValue().chars().filter(c -> c == '?').count();
		assertEquals(15, placeholders, "INSERT statement should have 15 bind placeholders");
	}
}
