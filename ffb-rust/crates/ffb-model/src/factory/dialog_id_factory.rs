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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_returns_known_id() {
        assert_eq!(DialogIdFactory::default().for_name("reRoll"), Some(DialogId::RE_ROLL));
        assert_eq!(DialogIdFactory::default().for_name("skillUse"), Some(DialogId::SKILL_USE));
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(DialogIdFactory::default().for_name("invalid"), None);
    }

    #[test]
    fn initialize_does_not_panic() {
        let mut f = DialogIdFactory::default();
        f.initialize();
    }

    #[test]
    fn for_name_a_second_known_variant() {
        assert_eq!(DialogIdFactory::default().for_name("blockRoll"), Some(DialogId::BLOCK_ROLL));
    }

    #[test]
    fn for_name_empty_string_returns_none() {
        assert_eq!(DialogIdFactory::default().for_name(""), None);
    }
}
