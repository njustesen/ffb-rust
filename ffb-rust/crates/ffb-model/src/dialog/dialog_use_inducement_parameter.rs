use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogUseInducementParameter.
/// Note: InducementType/Card serialized as String name (stubs not yet translated).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogUseInducementParameter {
    pub team_id: Option<String>,
    /// InducementType[] serialized as names.
    pub inducement_types: Vec<String>,
    /// Card[] serialized as names.
    pub cards: Vec<String>,
    pub player_id: Option<String>,
}

impl DialogUseInducementParameter {
    pub fn get_team_id(&self) -> Option<&str> { self.team_id.as_deref() }
    pub fn get_inducement_types(&self) -> &[String] { &self.inducement_types }
    pub fn get_cards(&self) -> &[String] { &self.cards }
    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
}

impl IDialogParameter for DialogUseInducementParameter {
    fn get_id(&self) -> DialogId { DialogId::USE_INDUCEMENT }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialog_id_is_use_inducement() {
        assert_eq!(DialogUseInducementParameter::default().get_id(), DialogId::USE_INDUCEMENT);
    }
    #[test]
    fn stores_team_id_and_cards() {
        let p = DialogUseInducementParameter {
            team_id: Some("home".into()),
            cards: vec!["CARD_A".into()],
            ..Default::default()
        };
        assert_eq!(p.get_team_id(), Some("home"));
        assert_eq!(p.get_cards().len(), 1);
    }
    #[test]
    fn default_is_sensible() {
        let p = DialogUseInducementParameter::default();
        assert!(p.get_team_id().is_none());
        assert!(p.get_inducement_types().is_empty());
        assert!(p.get_cards().is_empty());
        assert!(p.get_player_id().is_none());
    }
    #[test]
    fn stores_inducement_types_and_player_id() {
        let p = DialogUseInducementParameter {
            inducement_types: vec!["WIZARD".into(), "BRIBE".into()],
            player_id: Some("p5".into()),
            ..Default::default()
        };
        assert_eq!(p.get_inducement_types().len(), 2);
        assert_eq!(p.get_player_id(), Some("p5"));
    }
    #[test]
    fn player_id_none_when_unset() {
        let p = DialogUseInducementParameter { player_id: None, ..Default::default() };
        assert!(p.get_player_id().is_none());
    }
}
