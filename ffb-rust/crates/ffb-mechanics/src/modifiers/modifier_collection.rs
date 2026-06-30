use crate::modifiers::modifier_type::ModifierType;

/// 1:1 translation of com.fumbbl.ffb.modifiers.ModifierCollection (Java generic abstract class).
/// Java: ModifierCollection<C extends ModifierContext, V extends RollModifier<C>> implements IKeyedItem
///   - private Set<V> modifiers
///   - getKey() -> getClass().getSimpleName()
///   - protected add(V modifier)
///   - getModifiers(ModifierType) -> filter by type
///   - getModifiers() -> all modifiers
/// Rust: Each concrete collection (CatchModifierCollection, etc.) holds Vec<ConcreteModifier>
///       and implements these methods directly. This marker trait provides the common contract.
pub trait ModifierCollection {
    fn get_key(&self) -> &str;
}

/// Marker for types that expose a ModifierType field, mirroring Java RollModifier<C>.getType().
pub trait HasModifierType {
    fn get_type(&self) -> ModifierType;
}
