/// 1:1 translation of com.fumbbl.ffb.IDialogParameter (Java interface).
pub trait IDialogParameter {
    fn get_dialog_id(&self) -> &str;
}
