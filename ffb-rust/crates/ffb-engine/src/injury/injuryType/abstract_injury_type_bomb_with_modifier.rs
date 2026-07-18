// Placeholder for com.fumbbl.ffb.server.injury.injuryType.AbstractInjuryTypeBombWithModifier.
//
// The Java source does exist (ffb-server/.../injury/injuryType/AbstractInjuryTypeBombWithModifier.java):
// a shared `handleInjury()` template for the two concrete Bomb-with-modifier types. Its logic
// (skip the armor roll entirely for `placedProneCausesInjuryRoll` defenders i.e. Ball-and-Chain;
// otherwise roll armor and, if unbroken, add the dynamic `SpecialEffect.BOMB` armor modifier
// gated by the `bombUsesMb` game option and recompute; if broken, add per-skill injury
// modifiers plus — only when no special-effect armor modifier applied — the `SpecialEffect.BOMB`
// injury modifier, same gating) is inlined directly into each concrete type's `handle_injury()`
// (`injury_type_bomb_with_modifier.rs`, `injury_type_bomb_with_modifier_for_spp.rs`) rather than
// factored out into a shared Rust function, since Rust has no direct equivalent of the Java
// abstract-class-with-shared-method pattern here. This module is kept as a stub purely to
// preserve the 1:1 Java-package → Rust-module mapping.
