/// BB2025 card handlers.
/// Java: CardHandlerFactory uses @RulesCollection(Rules.COMMON) — no bb2025/cards/ directory exists
/// in the Java source. All editions share the same card handler set via Scanner.
/// Rust: re-export from bb2020::cards so factory initialization can reference either module.
pub use crate::inducements::bb2020::cards::chop_block_handler::ChopBlockHandler;
pub use crate::inducements::bb2020::cards::custard_pie_handler::CustardPieHandler;
pub use crate::inducements::bb2020::cards::distract_handler::DistractHandler;
pub use crate::inducements::bb2020::cards::force_shield_handler::ForceShieldHandler;
pub use crate::inducements::bb2020::cards::illegal_substitution_handler::IllegalSubstitutionHandler;
pub use crate::inducements::bb2020::cards::pit_trap_handler::PitTrapHandler;
pub use crate::inducements::bb2020::cards::rabbits_foot_handler::RabbitsFootHandler;
pub use crate::inducements::bb2020::cards::witch_brew_handler::WitchBrewHandler;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inducements::card_handler::CardHandler;

    #[test]
    fn bb2025_chop_block_handler_is_accessible() {
        let h: Box<dyn CardHandler> = Box::new(ChopBlockHandler::new());
        assert_eq!(h.handler_key_name(), "CHOP_BLOCK");
    }

    #[test]
    fn bb2025_custard_pie_handler_is_accessible() {
        let h: Box<dyn CardHandler> = Box::new(CustardPieHandler::new());
        assert_eq!(h.handler_key_name(), "CUSTARD_PIE");
    }

    #[test]
    fn bb2025_all_eight_handlers_construct() {
        let handlers: Vec<Box<dyn CardHandler>> = vec![
            Box::new(ChopBlockHandler::new()),
            Box::new(CustardPieHandler::new()),
            Box::new(DistractHandler::new()),
            Box::new(ForceShieldHandler::new()),
            Box::new(IllegalSubstitutionHandler::new()),
            Box::new(PitTrapHandler::new()),
            Box::new(RabbitsFootHandler::new()),
            Box::new(WitchBrewHandler::new()),
        ];
        assert_eq!(handlers.len(), 8);
    }
}
