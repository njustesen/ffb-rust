/// 1:1 translation of com.fumbbl.ffb.IIconProperty (Java interface).
pub trait IIconProperty {
    fn get_icon_path(&self) -> &str;
}
