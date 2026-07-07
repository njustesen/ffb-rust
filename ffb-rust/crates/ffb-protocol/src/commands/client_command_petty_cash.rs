/// 1:1 translation of ClientCommandPettyCash (Java field: fPettyCash).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandPettyCash {
    pub petty_cash: i32,
}

impl ClientCommandPettyCash {
    pub fn new(petty_cash: i32) -> Self {
        Self { petty_cash }
    }

    pub fn get_petty_cash(&self) -> i32 {
        self.petty_cash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_value() {
        let cmd = ClientCommandPettyCash::new(50_000);
        assert_eq!(cmd.get_petty_cash(), 50_000);
    }

    #[test]
    fn default_is_zero() {
        let cmd = ClientCommandPettyCash::default();
        assert_eq!(cmd.get_petty_cash(), 0);
    }

    #[test]
    fn negative_value_stored() {
        let cmd = ClientCommandPettyCash::new(-1000);
        assert_eq!(cmd.get_petty_cash(), -1000);
    }


    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandPettyCash::default();
        assert!(!format!("{cmd:?}").is_empty());
    }
}
