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

    /// Java: `transform()` returns `new DialogSelectKeywordParameter(playerId, keywords,
    /// keywordChoiceMode, 1, 1)` — min/max select are hardcoded to 1, not carried over
    /// from the original instance.
    pub fn transform_typed(&self) -> Self {
        DialogSelectKeywordParameter {
            keywords: self.keywords.clone(),
            player_id: self.player_id.clone(),
            keyword_choice_mode: self.keyword_choice_mode.clone(),
            min_select: 1,
            max_select: 1,
        }
    }
}

impl IDialogParameter for DialogSelectKeywordParameter {
    fn get_id(&self) -> DialogId { DialogId::SELECT_KEYWORD }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.transform_typed()) }
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

    #[test]
    fn default_is_sensible() {
        let p = DialogSelectKeywordParameter::default();
        assert!(p.get_keywords().is_empty());
        assert!(p.get_player_id().is_none());
        assert!(p.get_keyword_choice_mode().is_none());
        assert_eq!(p.get_min_select(), 0);
        assert_eq!(p.get_max_select(), 0);
    }

    #[test]
    fn keywords_and_player_id_stored() {
        let p = DialogSelectKeywordParameter {
            keywords: vec!["PASS".into(), "RUSH".into()],
            player_id: Some("p1".into()),
            ..Default::default()
        };
        assert_eq!(p.get_keywords().len(), 2);
        assert_eq!(p.get_player_id(), Some("p1"));
    }

    #[test]
    fn keyword_choice_mode_accessor() {
        let p = DialogSelectKeywordParameter {
            keyword_choice_mode: Some("SINGLE".into()),
            ..Default::default()
        };
        assert_eq!(p.get_keyword_choice_mode(), Some("SINGLE"));
    }

    #[test]
    fn transform_resets_min_max_select_to_one() {
        // Java's transform() always hardcodes minSelect/maxSelect to 1, regardless of
        // the original values — it does NOT just clone the instance.
        let p = DialogSelectKeywordParameter {
            player_id: Some("p1".into()),
            keywords: vec!["PASS".into()],
            keyword_choice_mode: Some("MULTIPLE".into()),
            min_select: 2,
            max_select: 5,
        };
        let transformed = p.transform_typed();
        assert_eq!(transformed.get_min_select(), 1);
        assert_eq!(transformed.get_max_select(), 1);
        assert_eq!(transformed.get_player_id(), Some("p1"));
        assert_eq!(transformed.get_keywords(), &["PASS".to_string()]);
    }
}
