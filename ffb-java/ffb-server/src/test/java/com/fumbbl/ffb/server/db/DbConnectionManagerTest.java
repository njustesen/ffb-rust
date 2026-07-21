package com.fumbbl.ffb.server.db;

import com.fumbbl.ffb.server.FantasyFootballServer;
import com.fumbbl.ffb.server.ServerMode;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.lenient;

/**
 * Mirrors the Rust {@code db_connection_manager} tests.
 *
 * <p>Divergence from Rust: the Rust {@code use_mysql_dialect} recognises both
 * "mysql" and "mariadb" (case-insensitively) whereas the Java
 * {@link DbConnectionManager#useMysqlDialect()} only accepts "mysql"
 * ({@code "mysql".equalsIgnoreCase(fDbType)}). The Java behaviour is asserted here.
 *
 * <p>The Rust pool/init tests ({@code init_pool}, {@code pool_ready}) are
 * mysql_async-specific and have no Java counterpart (Java opens JDBC connections
 * lazily via DriverManager), so they are not ported.
 */
@ExtendWith(MockitoExtension.class)
class DbConnectionManagerTest {

	@Mock
	private FantasyFootballServer server;

	@Test
	void useMysqlDialectCaseInsensitive() {
		DbConnectionManager m = new DbConnectionManager(server);
		m.setDbType("mysql");
		assertTrue(m.useMysqlDialect());
		m.setDbType("MYSQL");
		assertTrue(m.useMysqlDialect());
		// Java, unlike Rust, does not treat mariadb as the mysql dialect.
		m.setDbType("mariadb");
		assertFalse(m.useMysqlDialect());
		m.setDbType("other");
		assertFalse(m.useMysqlDialect());
	}

	@Test
	void isStandaloneReflectsServerMode() {
		lenient().when(server.getMode()).thenReturn(ServerMode.FUMBBL);
		DbConnectionManager m = new DbConnectionManager(server);
		assertFalse(m.isStandalone());
	}
}
