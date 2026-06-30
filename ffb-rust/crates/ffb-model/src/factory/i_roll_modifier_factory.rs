/// 1:1 translation of com.fumbbl.ffb.factory.IRollModifierFactory.
pub trait IRollModifierFactory<T> {
    fn for_name(&self, name: &str) -> Option<T>;
}
