package com.fumbbl.ffb.server.db;

import com.fumbbl.ffb.server.DebugLog;
import com.fumbbl.ffb.server.FantasyFootballServer;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.sql.SQLException;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.doNothing;
import static org.mockito.Mockito.doThrow;
import static org.mockito.Mockito.lenient;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

/**
 * Mirrors the Rust {@code db_transaction} tests.
 */
@ExtendWith(MockitoExtension.class)
class DbTransactionTest {

	@Mock
	private FantasyFootballServer server;

	@Mock
	private DbUpdateFactory updateFactory;

	@Mock
	private DebugLog debugLog;

	@Test
	void addIncreasesSize() throws SQLException {
		DbTransaction t = new DbTransaction();
		t.add(okParameter());
		assertEquals(1, t.size());
	}

	@Test
	void executeUpdateCommitsAndAccumulatesRowsOnSuccess() throws SQLException {
		when(server.getDbUpdateFactory()).thenReturn(updateFactory);
		DbTransaction t = new DbTransaction();
		t.add(okParameter());
		t.add(okParameter());
		t.executeUpdate(server);
		assertEquals(2, t.getUpdatedRows());
		verify(updateFactory).commit();
	}

	@Test
	void executeUpdateRollsBackAndZeroesRowsOnFailure() throws SQLException {
		when(server.getDbUpdateFactory()).thenReturn(updateFactory);
		when(server.getDebugLog()).thenReturn(debugLog);
		DbTransaction t = new DbTransaction();
		t.add(okParameter());
		t.add(failingParameter());
		t.executeUpdate(server);
		assertEquals(0, t.getUpdatedRows());
		verify(updateFactory).rollback();
	}

	private IDbUpdateParameter okParameter() throws SQLException {
		IDbUpdateParameter p = org.mockito.Mockito.mock(IDbUpdateParameter.class);
		lenient().doNothing().when(p).executeUpdate(any());
		lenient().when(p.getUpdatedRows()).thenReturn(1);
		// On the failure path DbTransaction logs every already-processed parameter's
		// statement, so an ok parameter must also expose one.
		DbUpdateStatement statement = org.mockito.Mockito.mock(DbUpdateStatement.class);
		lenient().when(statement.toString(any())).thenReturn("stmt");
		lenient().when(p.getDbUpdateStatement(any())).thenReturn(statement);
		return p;
	}

	private IDbUpdateParameter failingParameter() throws SQLException {
		IDbUpdateParameter p = org.mockito.Mockito.mock(IDbUpdateParameter.class);
		doThrow(new SQLException("boom")).when(p).executeUpdate(any());
		DbUpdateStatement statement = org.mockito.Mockito.mock(DbUpdateStatement.class);
		lenient().when(statement.toString(any())).thenReturn("stmt");
		lenient().when(p.getDbUpdateStatement(any())).thenReturn(statement);
		return p;
	}
}
