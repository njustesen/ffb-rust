use crate::enums::KickoffResult;

/// 1:1 translation of com.fumbbl.ffb.factory.KickoffResultFactory.
pub struct KickoffResultFactory;

impl Default for KickoffResultFactory {
    fn default() -> Self { Self }
}

impl KickoffResultFactory {
    pub fn for_name(&self, name: &str) -> Option<KickoffResult> {
        KickoffResult::from_name(name)
    }

    pub fn initialize(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_result() {
        assert_eq!(KickoffResultFactory::default().for_name("Blitz"), Some(KickoffResult::Blitz));
        assert_eq!(KickoffResultFactory::default().for_name("Riot"), Some(KickoffResult::Riot));
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(KickoffResultFactory::default().for_name("invalid"), None);
    }

    #[test]
    fn initialize_does_not_panic() {
        let mut f = KickoffResultFactory::default();
        f.initialize();
    }

    #[test]
    fn for_name_a_second_known_variant() {
        assert_eq!(KickoffResultFactory::default().for_name("High Kick"), Some(KickoffResult::HighKick));
    }

    #[test]
    fn for_name_empty_string_returns_none() {
        assert_eq!(KickoffResultFactory::default().for_name(""), None);
    }
}
