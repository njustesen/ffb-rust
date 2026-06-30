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
