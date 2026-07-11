/// 1:1 translation of com.fumbbl.ffb.server.db.DefaultDbUpdateParameter.
use crate::db::i_db_update_parameter::IDbUpdateParameter;

pub struct DefaultDbUpdateParameter {
    updated_rows: i32,
}

impl DefaultDbUpdateParameter {
    pub fn new() -> Self {
        Self { updated_rows: 0 }
    }
}

impl Default for DefaultDbUpdateParameter {
    fn default() -> Self {
        Self::new()
    }
}

impl IDbUpdateParameter for DefaultDbUpdateParameter {
    fn get_updated_rows(&self) -> i32 {
        self.updated_rows
    }

    /// Java: `fUpdatedRows = getDbUpdateStatement(pServer).execute(this);`
    /// `getDbUpdateStatement()` is abstract in Java — `DefaultDbUpdateParameter` is never
    /// instantiated directly there, only extended by concrete parameter types that override
    /// it. Rust has no generic "resolve the bound DbUpdateStatement for this parameter"
    /// dispatch mechanism yet (each concrete statement's execute() has a distinct signature),
    /// so this base type returns a real, non-panicking error instead of the Java abstract
    /// method's implicit "must be overridden" contract.
    fn execute_update(&mut self) -> Result<(), String> {
        Err(
            "DefaultDbUpdateParameter has no bound DbUpdateStatement — override in a concrete parameter type"
                .to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let p = DefaultDbUpdateParameter::new();
        assert_eq!(p.get_updated_rows(), 0);
    }

    #[test]
    fn execute_update_returns_err_without_panicking() {
        let mut p = DefaultDbUpdateParameter::new();
        let result = p.execute_update();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no bound DbUpdateStatement"));
    }
}
