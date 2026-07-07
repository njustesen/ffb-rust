/// 1:1 translation of com.fumbbl.ffb.net.commands.ClientCommandBlockOrReRollChoiceForTarget.
use ffb_model::enums::ReRollSource;

#[derive(Debug, Clone)]
pub struct ClientCommandBlockOrReRollChoiceForTarget {
    /// Java: `targetId`
    pub target_id: Option<String>,
    /// Java: `selectedIndex` — defaults to -1 in Java
    pub selected_index: i32,
    /// Java: `proIndex`
    pub pro_index: i32,
    /// Java: `reRollSource`
    pub re_roll_source: Option<ReRollSource>,
    /// Java: `anyDiceIndexes`
    pub any_dice_indexes: Vec<i32>,
}

impl Default for ClientCommandBlockOrReRollChoiceForTarget {
    fn default() -> Self {
        Self {
            target_id: None,
            selected_index: -1,
            pro_index: 0,
            re_roll_source: None,
            any_dice_indexes: vec![],
        }
    }
}

impl ClientCommandBlockOrReRollChoiceForTarget {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java: `getTargetId()`
    pub fn get_target_id(&self) -> Option<&str> {
        self.target_id.as_deref()
    }

    /// Java: `getSelectedIndex()`
    pub fn get_selected_index(&self) -> i32 {
        self.selected_index
    }

    /// Java: `getProIndex()`
    pub fn get_pro_index(&self) -> i32 {
        self.pro_index
    }

    /// Java: `getReRollSource()`
    pub fn get_re_roll_source(&self) -> Option<&ReRollSource> {
        self.re_roll_source.as_ref()
    }

    /// Java: `getAnyDiceIndexes()`
    pub fn get_any_dice_indexes(&self) -> &[i32] {
        &self.any_dice_indexes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_selected_index_is_minus_one() {
        let cmd = ClientCommandBlockOrReRollChoiceForTarget::new();
        assert_eq!(cmd.get_selected_index(), -1);
    }

    #[test]
    fn stores_target_id_and_any_dice_indexes() {
        let cmd = ClientCommandBlockOrReRollChoiceForTarget {
            target_id: Some("target_1".to_string()),
            selected_index: 2,
            pro_index: 1,
            re_roll_source: None,
            any_dice_indexes: vec![0, 2],
        };
        assert_eq!(cmd.get_target_id(), Some("target_1"));
        assert_eq!(cmd.get_selected_index(), 2);
        assert_eq!(cmd.get_any_dice_indexes(), &[0, 2]);
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandBlockOrReRollChoiceForTarget::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_roundtrip() {
        let cmd = ClientCommandBlockOrReRollChoiceForTarget::default();
        let _ = cmd.clone();
    }
}
