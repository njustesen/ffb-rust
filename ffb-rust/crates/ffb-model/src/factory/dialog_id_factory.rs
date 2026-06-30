use crate::dialog::dialog_id::DialogId;

/// 1:1 translation of com.fumbbl.ffb.factory.DialogIdFactory.
pub struct DialogIdFactory;

impl Default for DialogIdFactory {
    fn default() -> Self { Self }
}

impl DialogIdFactory {
    pub fn for_name(&self, name: &str) -> Option<DialogId> {
        DialogId::for_name(name)
    }

    pub fn initialize(&mut self) {}
}
