/// 1:1 translation of com.fumbbl.ffb.factory.INamedObjectFactory.
/// In Java this is a generic interface with `forName()` and `initialize()`.
/// In Rust, each factory implements this structurally (no dyn dispatch needed).
pub trait INamedObjectFactory<T> {
    fn for_name(&self, name: &str) -> Option<T>;
    fn initialize(&mut self);
}
