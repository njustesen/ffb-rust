use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.model.property.PassingProperty.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PassingProperty {
    pub modifier: i32,
}

impl PassingProperty {
    pub fn new(modifier: i32) -> Self { Self { modifier } }
    pub fn get_modifier(&self) -> i32 { self.modifier }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_modifier_is_zero() {
        assert_eq!(PassingProperty::default().modifier, 0);
    }

    #[test]
    fn new_sets_modifier() {
        let p = PassingProperty::new(2);
        assert_eq!(p.get_modifier(), 2);
    }
}
