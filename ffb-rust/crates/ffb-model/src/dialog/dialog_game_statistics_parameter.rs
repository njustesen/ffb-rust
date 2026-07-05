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
}
