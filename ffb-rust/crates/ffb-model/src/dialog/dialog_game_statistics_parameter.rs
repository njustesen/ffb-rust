use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogGameStatisticsParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogGameStatisticsParameter;

impl IDialogParameter for DialogGameStatisticsParameter {
    fn get_id(&self) -> DialogId { DialogId::GAME_STATISTICS }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialog_id_is_game_statistics() {
        assert_eq!(DialogGameStatisticsParameter.get_id(), DialogId::GAME_STATISTICS);
    }
    #[test]
    fn transform_preserves_id() {
        assert_eq!(DialogGameStatisticsParameter.transform().get_id(), DialogId::GAME_STATISTICS);
    }

    #[test]
    fn default_is_sensible() {
        let p = DialogGameStatisticsParameter::default();
        assert_eq!(p.get_id(), DialogId::GAME_STATISTICS);
    }

    #[test]
    fn serde_round_trip() {
        let p = DialogGameStatisticsParameter;
        let json = serde_json::to_string(&p).unwrap();
        let back: DialogGameStatisticsParameter = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get_id(), DialogId::GAME_STATISTICS);
    }

    #[test]
    fn clone_preserves_id() {
        let p = DialogGameStatisticsParameter;
        let cloned = p.clone();
        assert_eq!(cloned.get_id(), DialogId::GAME_STATISTICS);
    }
}
