use super::dialog_id::DialogId;

/// 1:1 translation of com.fumbbl.ffb.IDialogParameter.
pub trait IDialogParameter: Send + Sync {
    fn get_id(&self) -> DialogId;
    fn transform(&self) -> Box<dyn IDialogParameter>;
}
