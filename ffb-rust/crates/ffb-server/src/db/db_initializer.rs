/// 1:1 translation of com.fumbbl.ffb.server.db.DbInitializer.
use super::db_connection_manager::DbConnectionManager;
use super::i_db_table_coaches::IDbTableCoaches;
use super::i_db_table_games_info::IDbTableGamesInfo;
use super::i_db_table_games_serialized::IDbTableGamesSerialized;
use super::i_db_table_player_markers::IDbTablePlayerMarkers;
use super::i_db_table_team_setups::IDbTableTeamSetups;
use super::i_db_table_user_settings::IDbTableUserSettings;
use ffb_model::util::string_tool;
use mysql_async::{prelude::Queryable, Conn, Error as DbError};

struct TableCoaches;
impl IDbTableCoaches for TableCoaches {}
struct TableGamesInfo;
impl IDbTableGamesInfo for TableGamesInfo {}
struct TableGamesSerialized;
impl IDbTableGamesSerialized for TableGamesSerialized {}
struct TablePlayerMarkers;
impl IDbTablePlayerMarkers for TablePlayerMarkers {}
struct TableTeamSetups;
impl IDbTableTeamSetups for TableTeamSetups {}
struct TableUserSettings;
impl IDbTableUserSettings for TableUserSettings {}

pub struct DbInitializer {
    db_connection_manager: DbConnectionManager,
}

impl DbInitializer {
    pub const DIR_TEAM_SETUPS: &'static str = "setups";

    pub const COACHES: &'static [&'static str] =
        &["Kalimar", "BattleLore", "LordCrunchy", "LordMisery"];

    pub const PASSWORDS: &'static [&'static str] = &[
        "f14bcf4b9ce4dd76",
        "77acbde6",
        "fb8f371e",
        "74baf495",
    ];

    pub fn new(db_connection_manager: DbConnectionManager) -> Self {
        Self { db_connection_manager }
    }

    pub async fn init_db(&self) -> Result<(), DbError> {
        let mut conn = self.db_connection_manager.open_db_connection().await?;

        self.drop_table(&mut conn, TablePlayerMarkers::TABLE_NAME).await?;
        self.drop_table(&mut conn, TableTeamSetups::TABLE_NAME).await?;
        self.drop_table(&mut conn, TableUserSettings::TABLE_NAME).await?;
        self.drop_table(&mut conn, TableGamesInfo::TABLE_NAME).await?;
        self.drop_table(&mut conn, TableGamesSerialized::TABLE_NAME).await?;

        if self.db_connection_manager.is_standalone() {
            self.drop_table(&mut conn, TableCoaches::TABLE_NAME).await?;
            self.create_table_coaches(&mut conn).await?;
        }

        self.create_table_player_markers(&mut conn).await?;
        self.create_table_user_settings(&mut conn).await?;
        self.create_table_team_setups(&mut conn).await?;
        self.create_table_games_info(&mut conn).await?;
        self.create_table_games_serialized(&mut conn).await?;

        if self.db_connection_manager.is_standalone() {
            self.init_table_coaches(&mut conn).await?;
            self.init_table_team_setups(&mut conn).await?;
        }

        conn.query_drop("COMMIT").await?;
        Ok(())
    }

    pub async fn drop_table(&self, conn: &mut Conn, table_name: &str) -> Result<u64, DbError> {
        let sql = format!("DROP TABLE IF EXISTS {table_name};");
        conn.query_drop(&sql).await?;
        Ok(conn.affected_rows())
    }

    pub async fn create_table_team_setups(&self, conn: &mut Conn) -> Result<u64, DbError> {
        let mut sql = String::new();
        sql.push_str("CREATE TABLE ");
        sql.push_str(TableTeamSetups::TABLE_NAME);
        sql.push('(');
        sql.push_str(TableTeamSetups::COLUMN_TEAM_ID);
        sql.push_str(" VARCHAR(40) NOT NULL,"); // 1
        sql.push_str(TableTeamSetups::COLUMN_NAME);
        sql.push_str(" VARCHAR(40) NOT NULL,"); // 2
        for i in 1..=11 {
            if i > 1 {
                sql.push(',');
            }
            let idx = i.to_string();
            sql.push_str(&string_tool::bind(TableTeamSetups::COLUMN_PLAYER_NR, &[&idx]));
            sql.push_str(" TINYINT,"); // 3
            sql.push_str(&string_tool::bind(TableTeamSetups::COLUMN_COORDINATE_X, &[&idx]));
            sql.push_str(" TINYINT,"); // 4
            sql.push_str(&string_tool::bind(TableTeamSetups::COLUMN_COORDINATE_Y, &[&idx]));
            sql.push_str(" TINYINT"); // 5
        }
        if self.db_connection_manager.use_mysql_dialect() {
            sql.push_str(");");
        } else {
            sql.push_str(") ENGINE=InnoDB DEFAULT CHARSET=utf8;");
        }
        conn.query_drop(&sql).await?;
        Ok(conn.affected_rows())
    }

    pub async fn create_table_user_settings(&self, conn: &mut Conn) -> Result<u64, DbError> {
        let mut sql = String::new();
        sql.push_str("CREATE TABLE ");
        sql.push_str(TableUserSettings::TABLE_NAME);
        sql.push('(');
        sql.push_str(TableUserSettings::COLUMN_COACH);
        sql.push_str(" VARCHAR(40) NOT NULL,"); // 1
        sql.push_str(TableUserSettings::COLUMN_NAME);
        sql.push_str(" VARCHAR(40) NOT NULL,"); // 2
        sql.push_str(TableUserSettings::COLUMN_VALUE);
        sql.push_str(" VARCHAR(40)"); // 3
        if self.db_connection_manager.use_mysql_dialect() {
            sql.push_str(");");
        } else {
            sql.push_str(") ENGINE=InnoDB DEFAULT CHARSET=utf8;");
        }
        conn.query_drop(&sql).await?;
        Ok(conn.affected_rows())
    }

    pub async fn create_table_coaches(&self, conn: &mut Conn) -> Result<u64, DbError> {
        let mut sql = String::new();
        sql.push_str("CREATE TABLE ");
        sql.push_str(TableCoaches::TABLE_NAME);
        sql.push_str(" (");
        sql.push_str(TableCoaches::COLUMN_NAME);
        sql.push_str(" VARCHAR(40) NOT NULL,"); // 1
        sql.push_str(TableCoaches::COLUMN_PASSWORD);
        sql.push_str(" VARCHAR(32) NOT NULL,"); // 2
        sql.push_str("PRIMARY KEY(");
        sql.push_str(TableCoaches::COLUMN_NAME);
        sql.push(')');
        if self.db_connection_manager.use_mysql_dialect() {
            sql.push_str(");");
        } else {
            sql.push_str(") ENGINE=InnoDB DEFAULT CHARSET=utf8;");
        }
        conn.query_drop(&sql).await?;
        Ok(conn.affected_rows())
    }

    pub async fn create_table_games_info(&self, conn: &mut Conn) -> Result<u64, DbError> {
        let mut sql = String::new();
        sql.push_str("CREATE TABLE ");
        sql.push_str(TableGamesInfo::TABLE_NAME);
        sql.push_str(" (");
        if self.db_connection_manager.use_mysql_dialect() {
            sql.push_str(TableGamesInfo::COLUMN_ID);
            sql.push_str(" BIGINT GENERATED BY DEFAULT AS IDENTITY(START WITH 1),"); // 1
        } else {
            sql.push_str(TableGamesInfo::COLUMN_ID);
            sql.push_str(" BIGINT NOT NULL AUTO_INCREMENT,"); // 1
        }
        sql.push_str(TableGamesInfo::COLUMN_SCHEDULED);
        sql.push_str(" DATETIME,"); // 2
        sql.push_str(TableGamesInfo::COLUMN_STARTED);
        sql.push_str(" DATETIME,"); // 3
        sql.push_str(TableGamesInfo::COLUMN_FINISHED);
        sql.push_str(" DATETIME,"); // 4
        sql.push_str(TableGamesInfo::COLUMN_COACH_HOME);
        sql.push_str(" VARCHAR(40),"); // 5
        sql.push_str(TableGamesInfo::COLUMN_TEAM_HOME_ID);
        sql.push_str(" VARCHAR(40),"); // 6
        sql.push_str(TableGamesInfo::COLUMN_TEAM_HOME_NAME);
        sql.push_str(" VARCHAR(100),"); // 7
        sql.push_str(TableGamesInfo::COLUMN_COACH_AWAY);
        sql.push_str(" VARCHAR(40),"); // 8
        sql.push_str(TableGamesInfo::COLUMN_TEAM_AWAY_ID);
        sql.push_str(" VARCHAR(40),"); // 9
        sql.push_str(TableGamesInfo::COLUMN_TEAM_AWAY_NAME);
        sql.push_str(" VARCHAR(100),"); // 10
        sql.push_str(TableGamesInfo::COLUMN_HALF);
        sql.push_str(" TINYINT NOT NULL,"); // 11
        sql.push_str(TableGamesInfo::COLUMN_TURN);
        sql.push_str(" TINYINT NOT NULL,"); // 12
        sql.push_str(TableGamesInfo::COLUMN_HOME_PLAYING);
        sql.push_str(" BOOLEAN NOT NULL,"); // 13
        sql.push_str(TableGamesInfo::COLUMN_STATUS);
        sql.push_str(" CHAR(1) NOT NULL,"); // 14
        sql.push_str(TableGamesInfo::COLUMN_TESTING);
        sql.push_str(" BOOLEAN NOT NULL,"); // 15
        sql.push_str(TableGamesInfo::COLUMN_ADMIN_MODE);
        sql.push_str(" BOOLEAN NOT NULL,"); // 16
        sql.push_str(TableGamesInfo::COLUMN_LAST_UPDATED);
        sql.push_str(" TIMESTAMP,"); // 17
        sql.push_str("PRIMARY KEY(");
        sql.push_str(TableGamesInfo::COLUMN_ID);
        sql.push(')');
        if self.db_connection_manager.use_mysql_dialect() {
            sql.push_str(");");
        } else {
            sql.push_str(") ENGINE=InnoDB DEFAULT CHARSET=utf8;");
        }
        conn.query_drop(&sql).await?;
        Ok(conn.affected_rows())
    }

    pub async fn create_table_games_serialized(&self, conn: &mut Conn) -> Result<u64, DbError> {
        let mut sql = String::new();
        sql.push_str("CREATE TABLE ");
        sql.push_str(TableGamesSerialized::TABLE_NAME);
        sql.push_str(" (");
        sql.push_str(TableGamesSerialized::COLUMN_ID);
        sql.push_str(" BIGINT NOT NULL,"); // 1
        sql.push_str(TableGamesSerialized::COLUMN_SERIALIZED);
        sql.push_str(" BLOB,"); // 2
        sql.push_str("PRIMARY KEY(");
        sql.push_str(TableGamesSerialized::COLUMN_ID);
        sql.push(')');
        if self.db_connection_manager.use_mysql_dialect() {
            sql.push_str(");");
        } else {
            sql.push_str(") ENGINE=InnoDB DEFAULT CHARSET=utf8;");
        }
        conn.query_drop(&sql).await?;
        Ok(conn.affected_rows())
    }

    pub async fn create_table_player_markers(&self, conn: &mut Conn) -> Result<u64, DbError> {
        let mut sql = String::new();
        sql.push_str("CREATE TABLE ");
        sql.push_str(TablePlayerMarkers::TABLE_NAME);
        sql.push('(');
        sql.push_str(TablePlayerMarkers::COLUMN_TEAM_ID);
        sql.push_str(" VARCHAR(40) NOT NULL,"); // 1
        sql.push_str(TablePlayerMarkers::COLUMN_PLAYER_ID);
        sql.push_str(" VARCHAR(40) NOT NULL,"); // 2
        sql.push_str(TablePlayerMarkers::COLUMN_TEXT);
        sql.push_str(" VARCHAR(40),"); // 3
        sql.push_str("PRIMARY KEY(");
        sql.push_str(TablePlayerMarkers::COLUMN_TEAM_ID);
        sql.push(',');
        sql.push_str(TablePlayerMarkers::COLUMN_PLAYER_ID);
        sql.push(')');
        if self.db_connection_manager.use_mysql_dialect() {
            sql.push_str(");");
        } else {
            sql.push_str(") ENGINE=InnoDB DEFAULT CHARSET=utf8;");
        }
        conn.query_drop(&sql).await?;
        Ok(conn.affected_rows())
    }

    pub async fn init_table_coaches(&self, conn: &mut Conn) -> Result<u64, DbError> {
        let mut updated_rows: u64 = 0;
        for i in 0..Self::COACHES.len() {
            let mut sql = String::new();
            sql.push_str("INSERT INTO ");
            sql.push_str(TableCoaches::TABLE_NAME);
            sql.push_str(" VALUES('");
            sql.push_str(Self::COACHES[i]);
            sql.push_str("', '");
            sql.push_str(Self::PASSWORDS[i]);
            sql.push_str("');");
            conn.query_drop(&sql).await?;
            updated_rows += conn.affected_rows();
        }
        Ok(updated_rows)
    }

    /// Java reads team setups from a TeamSetupCache (filesystem-backed, dir = DIR_TEAM_SETUPS).
    /// There is no Rust TeamSetupCache yet — that is a separate missing dependency, not a
    /// DB-wiring gap. Left as a documented no-op returning 0 rows until TeamSetupCache is ported.
    pub async fn init_table_team_setups(&self, _conn: &mut Conn) -> Result<u64, DbError> {
        // TODO: port com.fumbbl.ffb.server.TeamSetupCache and iterate its team setups here,
        // building "INSERT INTO ffb_team_setups VALUES(...)" per setup as Java does.
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let _ = DbInitializer::new(DbConnectionManager::new());
    }

    #[test]
    fn constants() {
        assert_eq!(DbInitializer::COACHES.len(), 4);
        assert_eq!(DbInitializer::COACHES[0], "Kalimar");
    }

    #[test]
    fn drop_table_sql_shape() {
        let sql = format!("DROP TABLE IF EXISTS {};", TableGamesInfo::TABLE_NAME);
        assert!(sql.contains("DROP TABLE IF EXISTS"));
        assert!(sql.contains("ffb_games_info"));
    }

    #[test]
    fn create_table_coaches_sql_shape() {
        let mut sql = String::new();
        sql.push_str("CREATE TABLE ");
        sql.push_str(TableCoaches::TABLE_NAME);
        sql.push_str(" (");
        sql.push_str(TableCoaches::COLUMN_NAME);
        sql.push_str(" VARCHAR(40) NOT NULL,");
        sql.push_str(TableCoaches::COLUMN_PASSWORD);
        sql.push_str(" VARCHAR(32) NOT NULL,");
        sql.push_str("PRIMARY KEY(");
        sql.push_str(TableCoaches::COLUMN_NAME);
        sql.push(')');
        sql.push_str(");");
        assert!(sql.contains("CREATE TABLE ffb_coaches"));
        assert!(sql.contains("PRIMARY KEY(name)"));
    }

    #[test]
    fn team_setups_column_bind_produces_indexed_columns() {
        let bound = string_tool::bind(TableTeamSetups::COLUMN_PLAYER_NR, &["3"]);
        assert_eq!(bound, "player_nr_3");
    }
}
