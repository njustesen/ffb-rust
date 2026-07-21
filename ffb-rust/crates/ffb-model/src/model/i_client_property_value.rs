/// 1:1 translation of com.fumbbl.ffb.IClientPropertyValue (Java interface).
pub trait IClientPropertyValue {
    fn get_value(&self) -> &str;
}
