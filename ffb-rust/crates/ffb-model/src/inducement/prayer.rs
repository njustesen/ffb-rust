use crate::enums::InducementDuration;

/// Trait for prayer effects — 1:1 translation of Java Prayer interface.
pub trait Prayer {
    fn get_name(&self) -> &str;
    fn affects_both_teams(&self) -> bool;
    fn get_description(&self) -> &str;
    fn get_duration(&self) -> InducementDuration;
    fn event_message(&self) -> &str { "" }
    fn is_changing_player(&self) -> bool;
}
