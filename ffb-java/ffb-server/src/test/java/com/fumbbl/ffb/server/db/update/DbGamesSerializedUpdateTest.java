package com.fumbbl.ffb.server.db.update;

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

@ExtendWith(MockitoExtension.class)
class DbGamesSerializedUpdateTest {

	@Mock
	private FantasyFootballServer server;

	@Mock
	private Connection connection;

	@Mock
	private PreparedStatement statement;

	@Test
	void sqlHasSetColumn() throws SQLException {
		ArgumentCaptor<String> sqlCaptor = ArgumentCaptor.forClass(String.class);
		when(connection.prepareStatement(anyString())).thenReturn(statement);
		new DbGamesSerializedUpdate(server).prepare(connection);
		verify(connection).prepareStatement(sqlCaptor.capture());
		String sql = sqlCaptor.getValue();
		assertTrue(sql.contains("serialized=?"), "SQL should set the serialized column");
		assertTrue(sql.toUpperCase().contains("UPDATE"), "SQL should be an UPDATE statement");
	}
}
