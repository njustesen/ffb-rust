/// Translations of com.fumbbl.ffb.server.injury.modification package.
///
/// Java: abstract base class InjuryContextModification<T extends ModificationParams>
/// plus concrete implementations for each skill-based injury modifier.

pub mod injury_context_modification;
pub mod modification_params;
pub mod old_pro_modification_params;
pub mod av_or_inj_modification;
pub mod brutal_block_modification;
pub mod crushing_blow_modification;
pub mod ghostly_flames_modification;
pub mod master_assassin_modification;
pub mod old_pro_modification;
pub mod savage_mauling_modification;
pub mod bb2020;
pub mod bb2025;

pub use injury_context_modification::InjuryContextModification;
pub use modification_params::ModificationParams;
pub use old_pro_modification_params::OldProModificationParams;
