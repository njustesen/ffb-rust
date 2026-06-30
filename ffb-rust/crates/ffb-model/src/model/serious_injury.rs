use crate::model::injury_attribute::InjuryAttribute;

/// 1:1 translation of com.fumbbl.ffb.SeriousInjury (Java interface).
pub trait SeriousInjury {
    fn get_name(&self) -> &str;
    fn get_button_text(&self) -> &str;
    fn get_description(&self) -> &str;
    fn get_recovery(&self) -> &str;
    fn get_injury_attribute(&self) -> Option<InjuryAttribute>;
    fn is_dead(&self) -> bool;
    fn is_poison(&self) -> bool;
    fn show_si_roll(&self) -> bool;
}
