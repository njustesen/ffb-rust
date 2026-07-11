/// 1:1 translation of com.fumbbl.ffb.server.db.DbTransaction.
///
/// Java's `executeUpdate(FantasyFootballServer)` needs a live server to log SQL exceptions
/// and to reach `server.getDbUpdateFactory()` for commit/rollback. There is no
/// `FantasyFootballServer` struct in this phase, so this takes the `DbUpdateFactory`
/// directly instead (same object Java would reach via the server) and drops the
/// debug-log-on-error side effect (no DebugLog wiring yet in this phase).
use super::db_update_factory::DbUpdateFactory;

pub struct DbTransaction {
    updated_rows: i32,
    db_update_parameters: Vec<Box<dyn super::i_db_update_parameter::IDbUpdateParameter>>,
}

impl DbTransaction {
    pub fn new() -> Self {
        Self {
            updated_rows: 0,
            db_update_parameters: Vec::new(),
        }
    }

    pub fn add(&mut self, parameter: Box<dyn super::i_db_update_parameter::IDbUpdateParameter>) {
        self.db_update_parameters.push(parameter);
    }

    pub fn size(&self) -> usize {
        self.db_update_parameters.len()
    }

    pub fn get_updated_rows(&self) -> i32 {
        self.updated_rows
    }

    pub async fn execute_update(
        &mut self,
        update_factory: &mut DbUpdateFactory,
    ) -> Result<(), String> {
        self.updated_rows = 0;
        let mut do_commit = true;

        for parameter in self.db_update_parameters.iter_mut() {
            match parameter.execute_update() {
                Ok(()) => {
                    self.updated_rows += parameter.get_updated_rows();
                }
                Err(_err) => {
                    do_commit = false;
                    break;
                }
            }
        }

        if do_commit {
            update_factory
                .commit()
                .await
                .map_err(|e| e.to_string())?;
        } else {
            self.updated_rows = 0;
            update_factory
                .rollback()
                .await
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    }
}

impl Default for DbTransaction {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::db_connection_manager::DbConnectionManager;
    use crate::db::i_db_update_parameter::IDbUpdateParameter;

    struct OkParameter;
    impl IDbUpdateParameter for OkParameter {
        fn get_updated_rows(&self) -> i32 {
            1
        }
        fn execute_update(&mut self) -> Result<(), String> {
            Ok(())
        }
    }

    struct FailingParameter;
    impl IDbUpdateParameter for FailingParameter {
        fn get_updated_rows(&self) -> i32 {
            0
        }
        fn execute_update(&mut self) -> Result<(), String> {
            Err("boom".to_string())
        }
    }

    #[test]
    fn construct() {
        let t = DbTransaction::new();
        assert_eq!(t.size(), 0);
        assert_eq!(t.get_updated_rows(), 0);
    }

    #[test]
    fn add_increases_size() {
        let mut t = DbTransaction::new();
        t.add(Box::new(OkParameter));
        assert_eq!(t.size(), 1);
    }

    #[test]
    fn execute_update_commits_and_accumulates_rows_on_success() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let mut t = DbTransaction::new();
        t.add(Box::new(OkParameter));
        t.add(Box::new(OkParameter));
        let mut factory = DbUpdateFactory::new(DbConnectionManager::new());
        let result = rt.block_on(t.execute_update(&mut factory));
        assert!(result.is_ok());
        assert_eq!(t.get_updated_rows(), 2);
    }

    #[test]
    fn execute_update_rolls_back_and_zeroes_rows_on_failure() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let mut t = DbTransaction::new();
        t.add(Box::new(OkParameter));
        t.add(Box::new(FailingParameter));
        let mut factory = DbUpdateFactory::new(DbConnectionManager::new());
        let result = rt.block_on(t.execute_update(&mut factory));
        assert!(result.is_ok());
        assert_eq!(t.get_updated_rows(), 0);
    }
}
