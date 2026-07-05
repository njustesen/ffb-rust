/// Translation of com.fumbbl.ffb.server.factory.CardHandlerFactory.
///
/// Java: Set<CardHandler> populated by Scanner reflection.
/// Rust: explicit registration via initialize(rules). Lookup by name (case-insensitive)
/// or by card responsibility.
use ffb_model::enums::Rules;
use ffb_model::inducement::card::Card;
use crate::inducements::card_handler::CardHandler;

pub struct CardHandlerFactory {
    /// Java: Set<CardHandler> handlers
    handlers: Vec<Box<dyn CardHandler>>,
}

impl CardHandlerFactory {
    pub fn new() -> Self { Self { handlers: Vec::new() } }

    /// Java: initialize(Game game) — Scanner populates via @RulesCollection annotations.
    /// Rust: explicit registration per edition (all editions share the same 8 card types).
    pub fn initialize(&mut self, rules: Rules) {
        match rules {
            Rules::Bb2016 => {
                use crate::inducements::bb2016::cards::{
                    chop_block_handler::ChopBlockHandler,
                    custard_pie_handler::CustardPieHandler,
                    distract_handler::DistractHandler,
                    force_shield_handler::ForceShieldHandler,
                    illegal_substitution_handler::IllegalSubstitutionHandler,
                    pit_trap_handler::PitTrapHandler,
                    rabbits_foot_handler::RabbitsFootHandler,
                    witch_brew_handler::WitchBrewHandler,
                };
                self.add(Box::new(ChopBlockHandler::new()));
                self.add(Box::new(CustardPieHandler::new()));
                self.add(Box::new(DistractHandler::new()));
                self.add(Box::new(ForceShieldHandler::new()));
                self.add(Box::new(IllegalSubstitutionHandler::new()));
                self.add(Box::new(PitTrapHandler::new()));
                self.add(Box::new(RabbitsFootHandler::new()));
                self.add(Box::new(WitchBrewHandler::new()));
            }
            Rules::Bb2020 | Rules::Bb2025 => {
                // Java: @RulesCollection(COMMON) — BB2020 and BB2025 use identical handlers.
                use crate::inducements::bb2020::cards::{
                    chop_block_handler::ChopBlockHandler,
                    custard_pie_handler::CustardPieHandler,
                    distract_handler::DistractHandler,
                    force_shield_handler::ForceShieldHandler,
                    illegal_substitution_handler::IllegalSubstitutionHandler,
                    pit_trap_handler::PitTrapHandler,
                    rabbits_foot_handler::RabbitsFootHandler,
                    witch_brew_handler::WitchBrewHandler,
                };
                self.add(Box::new(ChopBlockHandler::new()));
                self.add(Box::new(CustardPieHandler::new()));
                self.add(Box::new(DistractHandler::new()));
                self.add(Box::new(ForceShieldHandler::new()));
                self.add(Box::new(IllegalSubstitutionHandler::new()));
                self.add(Box::new(PitTrapHandler::new()));
                self.add(Box::new(RabbitsFootHandler::new()));
                self.add(Box::new(WitchBrewHandler::new()));
            }
            _ => {}
        }
    }

    pub fn add(&mut self, handler: Box<dyn CardHandler>) {
        self.handlers.push(handler);
    }

    /// Java: forName(String pName) — case-insensitive name lookup.
    pub fn for_name(&self, name: &str) -> Option<&dyn CardHandler> {
        self.handlers.iter()
            .find(|h| h.get_name().eq_ignore_ascii_case(name))
            .map(|h| h.as_ref())
    }

    /// Java: forCard(Card card) — find handler responsible for this card.
    pub fn for_card(&self, card: &Card) -> Option<&dyn CardHandler> {
        self.handlers.iter()
            .find(|h| h.is_responsible(card))
            .map(|h| h.as_ref())
    }

    pub fn len(&self) -> usize { self.handlers.len() }
    pub fn is_empty(&self) -> bool { self.handlers.is_empty() }
}

impl Default for CardHandlerFactory {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;

    struct TestHandler { key: &'static str, name: &'static str }
    impl CardHandler for TestHandler {
        fn handler_key_name(&self) -> &'static str { self.key }
        fn get_name(&self) -> &'static str { self.name }
    }

    #[test]
    fn for_name_finds_case_insensitive() {
        let mut f = CardHandlerFactory::new();
        f.add(Box::new(TestHandler { key: "K", name: "MyHandler" }));
        assert!(f.for_name("myhandler").is_some());
        assert!(f.for_name("MYHANDLER").is_some());
    }

    #[test]
    fn for_name_miss_returns_none() {
        let f = CardHandlerFactory::new();
        assert!(f.for_name("Unknown").is_none());
    }

    #[test]
    fn for_card_finds_responsible_handler() {
        let mut f = CardHandlerFactory::new();
        f.add(Box::new(TestHandler { key: "THE_KEY", name: "TheHandler" }));
        let card = Card::new("Test Card", Some("THE_KEY"));
        assert!(f.for_card(&card).is_some());
    }

    #[test]
    fn for_card_no_match_returns_none() {
        let f = CardHandlerFactory::new();
        let card = Card::new("Test Card", Some("NO_HANDLER_KEY"));
        assert!(f.for_card(&card).is_none());
    }

    #[test]
    fn initialize_bb2016_registers_eight_handlers() {
        let mut f = CardHandlerFactory::new();
        f.initialize(Rules::Bb2016);
        assert_eq!(f.len(), 8);
    }

    #[test]
    fn initialize_bb2020_registers_eight_handlers() {
        let mut f = CardHandlerFactory::new();
        f.initialize(Rules::Bb2020);
        assert_eq!(f.len(), 8);
    }

    #[test]
    fn initialize_bb2025_registers_eight_handlers() {
        let mut f = CardHandlerFactory::new();
        f.initialize(Rules::Bb2025);
        assert_eq!(f.len(), 8);
    }

    #[test]
    fn for_card_finds_chop_block_after_initialize_bb2016() {
        let mut f = CardHandlerFactory::new();
        f.initialize(Rules::Bb2016);
        let card = Card::new("Chop Block", Some("CHOP_BLOCK"));
        assert!(f.for_card(&card).is_some());
    }

    #[test]
    fn for_card_finds_distract_after_initialize_bb2020() {
        let mut f = CardHandlerFactory::new();
        f.initialize(Rules::Bb2020);
        let card = Card::new("Distract", Some("DISTRACT"));
        assert!(f.for_card(&card).is_some());
    }

    #[test]
    fn for_card_finds_witch_brew_after_initialize_bb2025() {
        let mut f = CardHandlerFactory::new();
        f.initialize(Rules::Bb2025);
        let card = Card::new("Witch Brew", Some("WITCH_BREW"));
        assert!(f.for_card(&card).is_some());
    }

    #[test]
    fn initialize_does_not_panic() {
        let mut f = CardHandlerFactory::new();
        f.initialize(Rules::Bb2025);
        f.initialize(Rules::Bb2016);
    }
}
