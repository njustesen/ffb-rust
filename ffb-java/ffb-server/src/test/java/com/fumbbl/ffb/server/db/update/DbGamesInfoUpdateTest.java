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
class DbGamesInfoUpdateTest {

	@Mock
	private FantasyFootballServer server;

	@Mock
	private Connection connection;

	@Mock
	private PreparedStatement statement;

	@Test
	void sqlHasSetColumns() throws SQLException {
		ArgumentCaptor<String> sqlCaptor = ArgumentCaptor.forClass(String.class);
		when(connection.prepareStatement(anyString())).thenReturn(statement);
		new DbGamesInfoUpdate(server).prepare(connection);
		verify(connection).prepareStatement(sqlCaptor.capture());
		String sql = sqlCaptor.getValue();
		assertTrue(sql.contains("scheduled=?"), sql);
		assertTrue(sql.contains("started=?"), sql);
		assertTrue(sql.contains("finished=?"), sql);
		assertTrue(sql.contains("coach_home=?"), sql);
		assertTrue(sql.contains("team_home_id=?"), sql);
		assertTrue(sql.contains("team_home_name=?"), sql);
		assertTrue(sql.contains("coach_away=?"), sql);
		assertTrue(sql.contains("team_away_id=?"), sql);
		assertTrue(sql.contains("team_away_name=?"), sql);
		assertTrue(sql.contains("half=?"), sql);
		assertTrue(sql.contains("turn=?"), sql);
		assertTrue(sql.contains("home_playing=?"), sql);
		assertTrue(sql.contains("status=?"), sql);
		assertTrue(sql.contains("testing=?"), sql);
		assertTrue(sql.contains("admin_mode=?"), sql);
	}
}
