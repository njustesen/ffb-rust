use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogSwarmingPlayersParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogSwarmingPlayersParameter {
    pub amount: i32,
    pub restrict_placement: bool,
}

impl DialogSwarmingPlayersParameter {
    pub fn get_amount(&self) -> i32 { self.amount }
    pub fn is_restrict_placement(&self) -> bool { self.restrict_placement }
}

impl IDialogParameter for DialogSwarmingPlayersParameter {
    fn get_id(&self) -> DialogId { DialogId::SWARMING }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialog_id_is_swarming() {
        assert_eq!(DialogSwarmingPlayersParameter::default().get_id(), DialogId::SWARMING);
    }
    #[test]
    fn stores_amount_and_restrict_placement() {
        let p = DialogSwarmingPlayersParameter { amount: 3, restrict_placement: true };
        assert_eq!(p.get_amount(), 3);
        assert!(p.is_restrict_placement());
    }
    #[test]
    fn default_is_sensible() {
        let p = DialogSwarmingPlayersParameter::default();
        assert_eq!(p.get_amount(), 0);
        assert!(!p.is_restrict_placement());
    }
    #[test]
    fn transform_preserves_id() {
        let p = DialogSwarmingPlayersParameter { amount: 5, restrict_placement: false };
        assert_eq!(p.transform().get_id(), DialogId::SWARMING);
    }
    #[test]
    fn zero_amount_unrestricted() {
        let p = DialogSwarmingPlayersParameter { amount: 0, restrict_placement: false };
        assert_eq!(p.get_amount(), 0);
        assert!(!p.is_restrict_placement());
    }
}
