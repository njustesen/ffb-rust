/// 1:1 translation of com.fumbbl.ffb.IClientProperty (Java interface).
pub trait IClientProperty {
    fn get_key(&self) -> &str;
}
