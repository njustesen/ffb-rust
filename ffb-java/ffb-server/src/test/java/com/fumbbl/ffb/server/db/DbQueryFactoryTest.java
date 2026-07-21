package com.fumbbl.ffb.server.db;

import com.fumbbl.ffb.server.FantasyFootballServer;
import com.fumbbl.ffb.server.ServerMode;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.mockito.Mockito.lenient;

/**
 * Mirrors the Rust {@code db_query_factory} tests.
 */
@ExtendWith(MockitoExtension.class)
class DbQueryFactoryTest {

	@Mock
	private FantasyFootballServer server;

	private DbQueryFactory factory() {
		lenient().when(server.getMode()).thenReturn(ServerMode.FUMBBL);
		return new DbQueryFactory(new DbConnectionManager(server));
	}

	@Test
	void construct() {
		assertNotNull(factory().getStatement(DbStatementId.GAMES_SERIALIZED_QUERY));
	}

	@Test
	void passwordForCoachQueryNotRegisteredWhenNotStandalone() {
		// mode is FUMBBL (not STANDALONE), so this statement is not registered.
		assertNull(factory().getStatement(DbStatementId.PASSWORD_FOR_COACH_QUERY));
	}

	@Test
	void unregisteredStatementReturnsNull() {
		// GAMES_INFO_UPDATE lives in the update factory, not the query factory.
		assertNull(factory().getStatement(DbStatementId.GAMES_INFO_UPDATE));
	}

	@Test
	void passwordForCoachQueryRegisteredWhenStandalone() {
		lenient().when(server.getMode()).thenReturn(ServerMode.STANDALONE);
		DbQueryFactory f = new DbQueryFactory(new DbConnectionManager(server));
		assertNotNull(f.getStatement(DbStatementId.PASSWORD_FOR_COACH_QUERY));
	}
}
