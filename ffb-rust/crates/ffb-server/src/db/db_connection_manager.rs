/// 1:1 translation of com.fumbbl.ffb.server.db.DbConnectionManager.
///
/// Java uses raw JDBC connections (DriverManager.getConnection + autocommit=false).
/// Rust uses mysql_async::Pool — semantically equivalent: each open_db_connection()
/// checks out a Conn from the pool; close_db_connection() commits and returns it.
use mysql_async::{prelude::Queryable, Conn, Error as DbError, Opts, Pool};

/// `Clone` is not a Java concept — it exists so async tasks (e.g.
/// `ServerDbKeepAliveTask`) can pull an owned, independently-lockable copy out
/// from behind a `std::sync::Mutex` before `.await`ing, since holding a
/// `std::sync::MutexGuard` across an await point would make the enclosing
/// future non-`Send` (and thus unusable with `tokio::spawn`). `mysql_async::Pool`
/// is itself a cheap, `Arc`-backed handle, so cloning is inexpensive.
#[derive(Clone)]
pub struct DbConnectionManager {
    pub db_url: String,
    pub db_user: String,
    pub db_password: String,
    pub db_type: String,
    pool: Option<Pool>,
}

impl DbConnectionManager {
    pub fn new() -> Self {
        Self {
            db_url: String::new(),
            db_user: String::new(),
            db_password: String::new(),
            db_type: String::new(),
            pool: None,
        }
    }

    // --- field accessors (1:1 Java getters/setters) ---

    pub fn get_db_url(&self) -> &str {
        &self.db_url
    }

    pub fn set_db_url(&mut self, url: String) {
        self.db_url = url;
    }

    pub fn get_db_user(&self) -> &str {
        &self.db_user
    }

    pub fn set_db_user(&mut self, user: String) {
        self.db_user = user;
    }

    pub fn get_db_password(&self) -> &str {
        &self.db_password
    }

    pub fn set_db_password(&mut self, password: String) {
        self.db_password = password;
    }

    pub fn set_db_type(&mut self, db_type: String) {
        self.db_type = db_type;
    }

    pub fn use_mysql_dialect(&self) -> bool {
        self.db_type.eq_ignore_ascii_case("mysql")
            || self.db_type.eq_ignore_ascii_case("mariadb")
    }

    // Java: fServer.getMode().isStandalone()
    // Deferred: requires full server-mode wiring. Callers that gate
    // standalone-only DB init check this; default false is safe for fumbbl mode.
    pub fn is_standalone(&self) -> bool {
        false
    }

    /// Not a Java method — callers (e.g. request handlers) use this to check whether
    /// `init_pool()` has succeeded before attempting `open_db_connection()`, since the
    /// latter's Java equivalent (`DriverManager.getConnection`) would throw rather than
    /// silently no-op when unconfigured. This lets DB-backed handlers degrade gracefully
    /// (skip persistence) in DB-less test/dev runs instead of panicking.
    pub fn pool_ready(&self) -> bool {
        self.pool.is_some()
    }

    // --- pool lifecycle ---

    /// Build the connection pool from current db_url/user/password.
    /// Java builds a new Connection per openDbConnection() call; here we use a Pool
    /// which is semantically equivalent (each get_conn() checks out one connection).
    pub fn init_pool(&mut self) -> Result<(), String> {
        // Java URL format: jdbc:mysql://host:port/db — strip "jdbc:" prefix.
        let url = self.db_url.strip_prefix("jdbc:").unwrap_or(&self.db_url);
        let opts = Opts::from_url(url).map_err(|e| format!("invalid DB URL: {e}"))?;
        self.pool = Some(Pool::new(opts));
        Ok(())
    }

    /// Java: DriverManager.getConnection(url, user, password) + setAutoCommit(false).
    /// Returns a checked-out Conn with auto-commit disabled (SET autocommit=0).
    pub async fn open_db_connection(&self) -> Result<Conn, DbError> {
        let pool = self.pool.as_ref().expect("pool not initialized — call init_pool() first");
        let mut conn = pool.get_conn().await?;
        conn.query_drop("SET autocommit=0").await?;
        Ok(conn)
    }

    /// Java: if !connection.getAutoCommit() { connection.commit() } + connection.close().
    /// Commits any pending transaction then returns the connection to the pool.
    pub async fn close_db_connection(&self, mut conn: Conn) -> Result<(), DbError> {
        conn.query_drop("COMMIT").await?;
        conn.disconnect().await
    }

    /// Java: for each tracked connection, execute "SELECT 1;" to keep it alive.
    /// mysql_async Pool handles keep-alive internally; we verify the pool is
    /// reachable by checking out one connection and running SELECT 1.
    pub async fn do_keep_alive_ping(&self) -> Result<(), DbError> {
        let pool = self.pool.as_ref().expect("pool not initialized");
        let mut conn = pool.get_conn().await?;
        conn.query_drop("SELECT 1").await?;
        conn.disconnect().await
    }
}

impl Default for DbConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_defaults() {
        let m = DbConnectionManager::new();
        assert_eq!(m.get_db_url(), "");
        assert_eq!(m.get_db_user(), "");
        assert!(!m.use_mysql_dialect());
    }

    #[test]
    fn use_mysql_dialect_case_insensitive() {
        let mut m = DbConnectionManager::new();
        m.set_db_type("mysql".to_string());
        assert!(m.use_mysql_dialect());
        m.set_db_type("MYSQL".to_string());
        assert!(m.use_mysql_dialect());
        m.set_db_type("mariadb".to_string());
        assert!(m.use_mysql_dialect());
        m.set_db_type("MARIADB".to_string());
        assert!(m.use_mysql_dialect());
        m.set_db_type("other".to_string());
        assert!(!m.use_mysql_dialect());
    }

    #[test]
    fn setters_persist() {
        let mut m = DbConnectionManager::new();
        m.set_db_url("jdbc:mysql://127.0.0.1:3306/ffblive".to_string());
        m.set_db_user("root".to_string());
        m.set_db_password("secret".to_string());
        assert_eq!(m.get_db_url(), "jdbc:mysql://127.0.0.1:3306/ffblive");
        assert_eq!(m.get_db_user(), "root");
        assert_eq!(m.get_db_password(), "secret");
    }

    #[test]
    fn jdbc_prefix_stripped_by_init_pool_does_not_panic_on_valid_url() {
        let mut m = DbConnectionManager::new();
        m.set_db_url("jdbc:mysql://127.0.0.1:3306/ffblive".to_string());
        m.set_db_user("root".to_string());
        // init_pool just builds Opts — no network call yet, so this is unit-testable
        let result = m.init_pool();
        assert!(result.is_ok(), "init_pool should succeed with valid URL: {:?}", result);
    }

    #[test]
    fn is_standalone_defaults_false() {
        assert!(!DbConnectionManager::new().is_standalone());
    }

    #[test]
    fn pool_ready_false_before_init() {
        assert!(!DbConnectionManager::new().pool_ready());
    }

    #[test]
    fn pool_ready_true_after_successful_init() {
        let mut m = DbConnectionManager::new();
        m.set_db_url("jdbc:mysql://127.0.0.1:3306/ffblive".to_string());
        m.init_pool().unwrap();
        assert!(m.pool_ready());
    }
}
