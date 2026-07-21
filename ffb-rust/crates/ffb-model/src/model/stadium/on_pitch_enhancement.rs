use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.model.stadium.OnPitchEnhancement.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OnPitchEnhancement {
    pub name: String,
    pub active: bool,
}

impl OnPitchEnhancement {
    pub fn new(name: String) -> Self { Self { name, active: false } }
    pub fn get_name(&self) -> &str { &self.name }
    pub fn is_active(&self) -> bool { self.active }
    pub fn set_active(&mut self, active: bool) { self.active = active; }
}
