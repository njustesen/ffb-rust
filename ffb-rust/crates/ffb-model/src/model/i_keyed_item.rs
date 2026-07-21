/// 1:1 translation of com.fumbbl.ffb.model.IKeyedItem (Java interface).
pub trait IKeyedItem {
    fn get_key(&self) -> &str;
}
