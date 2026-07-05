use crate::enums::PassResult;

/// 1:1 translation of com.fumbbl.ffb.factory.PassResultFactory.
pub struct PassResultFactory;

impl Default for PassResultFactory {
    fn default() -> Self { Self }
}

impl PassResultFactory {
    pub fn for_name(&self, name: &str) -> Option<PassResult> {
        PassResult::from_name(name)
    }

    pub fn initialize(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_result() {
        let f = PassResultFactory::default();
        assert_eq!(f.for_name("complete"), Some(PassResult::Complete));
        assert_eq!(f.for_name("fumble"), Some(PassResult::Fumble));
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(PassResultFactory::default().for_name("invalid"), None);
    }
}
