/// 1:1 translation of com.fumbbl.ffb.factory.IFactorySource.
/// Provides access to the factory registry and context.
/// Full implementation requires FactoryManager (Phase 1C).
pub trait IFactorySource {
    fn get_context(&self) -> &str;
}
