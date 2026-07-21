package com.fumbbl.ffb.server.db;

import com.fumbbl.ffb.server.FantasyFootballServer;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the Rust {@code db_update_factory} tests.
 *
 * <p>The Rust {@code commit_and_rollback_are_no_ops_without_open_connection} test
 * is not ported: the Java {@link DbUpdateFactory#commit()}/{@code rollback()}
 * dereference {@code fDbConnection}, which is null until {@code prepareStatements()}
 * has opened a connection, so they throw {@link NullPointerException} rather than
 * being safe no-ops. Exercising them requires a live JDBC connection.
 */
@ExtendWith(MockitoExtension.class)
class DbUpdateFactoryTest {

	@Mock
	private FantasyFootballServer server;

	private DbUpdateFactory factory() {
		return new DbUpdateFactory(new DbConnectionManager(server));
	}

	@Test
	void construct() {
		DbUpdateFactory f = factory();
		assertNotNull(f.getStatement(DbStatementId.GAMES_INFO_UPDATE));
		assertNotNull(f.getStatement(DbStatementId.TEAM_SETUPS_DELETE));
	}

	@Test
	void unregisteredStatementReturnsNull() {
		assertNull(factory().getStatement(DbStatementId.PASSWORD_FOR_COACH_QUERY));
	}
}
