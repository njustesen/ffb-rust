use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogSelectKeywordParameter.
/// Note: Keyword/KeywordChoiceMode serialized as String (stubs not yet translated).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogSelectKeywordParameter {
    pub keywords: Vec<String>,
    pub player_id: Option<String>,
    pub keyword_choice_mode: Option<String>,
    pub min_select: i32,
    pub max_select: i32,
}

impl DialogSelectKeywordParameter {
    pub fn get_keywords(&self) -> &[String] { &self.keywords }
    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_keyword_choice_mode(&self) -> Option<&str> { self.keyword_choice_mode.as_deref() }
    pub fn get_min_select(&self) -> i32 { self.min_select }
    pub fn get_max_select(&self) -> i32 { self.max_select }
}

impl IDialogParameter for DialogSelectKeywordParameter {
    fn get_id(&self) -> DialogId { DialogId::SELECT_KEYWORD }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialog_id_is_select_keyword() {
        assert_eq!(DialogSelectKeywordParameter::default().get_id(), DialogId::SELECT_KEYWORD);
    }
    #[test]
    fn stores_min_max_select() {
        let p = DialogSelectKeywordParameter { min_select: 1, max_select: 3, ..Default::default() };
        assert_eq!(p.get_min_select(), 1);
        assert_eq!(p.get_max_select(), 3);
    }
}
