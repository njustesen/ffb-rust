package com.fumbbl.ffb.server.db.delete;

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
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class DbUserSettingsDeleteParameterTest {

	@Mock
	private FantasyFootballServer server;

	@Mock
	private Connection connection;

	@Mock
	private PreparedStatement statement;

	private String prepareSql() throws SQLException {
		ArgumentCaptor<String> sqlCaptor = ArgumentCaptor.forClass(String.class);
		when(connection.prepareStatement(anyString())).thenReturn(statement);
		new DbUserSettingsDelete(server).prepare(connection);
		verify(connection).prepareStatement(sqlCaptor.capture());
		return sqlCaptor.getValue();
	}

	@Test
	void getUpdatedRowsInitial() {
		DbUserSettingsDeleteParameter p = new DbUserSettingsDeleteParameter("coach1");
		assertEquals(0, p.getUpdatedRows());
	}

	@Test
	void sqlReferencesUserSettingsTable() throws SQLException {
		assertTrue(prepareSql().contains("ffb_user_settings"), "SQL must target ffb_user_settings table");
	}

	@Test
	void sqlIsDeleteByCoach() throws SQLException {
		String sql = prepareSql();
		assertTrue(sql.toUpperCase().startsWith("DELETE"), "SQL must be a DELETE statement");
		assertTrue(sql.contains("coach"), "SQL must filter by coach column");
	}
}
