use crate::model::SoundId;

/// 1:1 translation of com.fumbbl.ffb.factory.SoundIdFactory.
pub struct SoundIdFactory;

impl Default for SoundIdFactory {
    fn default() -> Self { SoundIdFactory }
}

impl SoundIdFactory {
    pub fn for_name(&self, name: &str) -> Option<SoundId> {
        SoundId::for_name(name)
    }

    pub fn initialize(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_sound_id() {
        let f = SoundIdFactory::default();
        assert_eq!(f.for_name("block"), Some(SoundId::BLOCK));
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(SoundIdFactory::default().for_name("invalid"), None);
    }

    #[test]
    fn for_name_touchdown_returns_some() {
        let f = SoundIdFactory::default();
        assert!(f.for_name("touchdown").is_some());
    }

    #[test]
    fn initialize_does_not_panic() {
        let mut f = SoundIdFactory::default();
        f.initialize();
    }
}
