/// 1:1 translation of com.fumbbl.ffb.INamedObject (Java interface).
pub trait INamedObject {
    fn get_name(&self) -> &str;
}
