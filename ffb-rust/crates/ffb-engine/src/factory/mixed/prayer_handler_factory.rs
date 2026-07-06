/// Translation of com.fumbbl.ffb.server.factory.mixed.PrayerHandlerFactory.
///
/// Applies to BB2020 and BB2025 rules. Java: Scanner reflection.
/// Rust: explicit registration via initialize(). Lookup by name or prayer name.
use ffb_model::enums::Rules;
use crate::inducements::mixed::prayers::prayer_handler::PrayerHandler;

pub struct PrayerHandlerFactory {
    /// Java: Set<PrayerHandler> handlers
    handlers: Vec<Box<dyn PrayerHandler>>,
}

impl PrayerHandlerFactory {
    pub fn new() -> Self { Self { handlers: Vec::new() } }

    /// Java: initialize(Game game) — Scanner populates via @RulesCollection annotations.
    /// Rust: explicit registration per edition.
    pub fn initialize(&mut self, rules: Rules) {
        match rules {
            Rules::Bb2020 => {
                use crate::inducements::bb2020::prayers::{
                    bad_habits_handler::BadHabitsHandler,
                    blessed_statue_of_nuffle_handler::BlessedStatueOfNuffleHandler,
                    fan_interaction_handler::FanInteractionHandler,
                    fouling_frenzy_handler::FoulingFrenzyHandler,
                    friends_with_the_ref_handler::FriendsWithTheRefHandler,
                    greasy_cleats_handler::GreasyCleatsHandler,
                    intensive_training_handler::IntensiveTrainingHandler,
                    iron_man_handler::IronManHandler,
                    knuckle_dusters_handler::KnuckleDustersHandler,
                    moles_under_the_pitch_handler::MolesUnderThePitchHandler,
                    necessary_violence_handler::NecessaryViolenceHandler,
                    perfect_passing_handler::PerfectPassingHandler,
                    stiletto_handler::StilettoHandler,
                    throw_a_rock_handler::ThrowARockHandler,
                    treacherous_trapdoor_handler::TreacherousTrapdoorHandler,
                    under_scrutiny_handler::UnderScrutinyHandler,
                };
                self.add(Box::new(BadHabitsHandler::new()));
                self.add(Box::new(BlessedStatueOfNuffleHandler::new()));
                self.add(Box::new(FanInteractionHandler::new()));
                self.add(Box::new(FoulingFrenzyHandler::new()));
                self.add(Box::new(FriendsWithTheRefHandler::new()));
                self.add(Box::new(GreasyCleatsHandler::new()));
                self.add(Box::new(IntensiveTrainingHandler::new()));
                self.add(Box::new(IronManHandler::new()));
                self.add(Box::new(KnuckleDustersHandler::new()));
                self.add(Box::new(MolesUnderThePitchHandler::new()));
                self.add(Box::new(NecessaryViolenceHandler::new()));
                self.add(Box::new(PerfectPassingHandler::new()));
                self.add(Box::new(StilettoHandler::new()));
                self.add(Box::new(ThrowARockHandler::new()));
                self.add(Box::new(TreacherousTrapdoorHandler::new()));
                self.add(Box::new(UnderScrutinyHandler::new()));
            }
            Rules::Bb2025 => {
                use crate::inducements::bb2025::prayers::{
                    bad_habits_handler::BadHabitsHandler,
                    blessed_statue_of_nuffle_handler::BlessedStatueOfNuffleHandler,
                    dazzling_catching_handler::DazzlingCatchingHandler,
                    fan_interaction_handler::FanInteractionHandler,
                    fouling_frenzy_handler::FoulingFrenzyHandler,
                    friends_with_the_ref_handler::FriendsWithTheRefHandler,
                    greasy_cleats_handler::GreasyCleatsHandler,
                    intensive_training_handler::IntensiveTrainingHandler,
                    iron_man_handler::IronManHandler,
                    knuckle_dusters_handler::KnuckleDustersHandler,
                    moles_under_the_pitch_handler::MolesUnderThePitchHandler,
                    perfect_passing_handler::PerfectPassingHandler,
                    stiletto_handler::StilettoHandler,
                    throw_a_rock_handler::ThrowARockHandler,
                    treacherous_trapdoor_handler::TreacherousTrapdoorHandler,
                    under_scrutiny_handler::UnderScrutinyHandler,
                };
                self.add(Box::new(BadHabitsHandler::new()));
                self.add(Box::new(BlessedStatueOfNuffleHandler::new()));
                self.add(Box::new(DazzlingCatchingHandler::new()));
                self.add(Box::new(FanInteractionHandler::new()));
                self.add(Box::new(FoulingFrenzyHandler::new()));
                self.add(Box::new(FriendsWithTheRefHandler::new()));
                self.add(Box::new(GreasyCleatsHandler::new()));
                self.add(Box::new(IntensiveTrainingHandler::new()));
                self.add(Box::new(IronManHandler::new()));
                self.add(Box::new(KnuckleDustersHandler::new()));
                self.add(Box::new(MolesUnderThePitchHandler::new()));
                self.add(Box::new(PerfectPassingHandler::new()));
                self.add(Box::new(StilettoHandler::new()));
                self.add(Box::new(ThrowARockHandler::new()));
                self.add(Box::new(TreacherousTrapdoorHandler::new()));
                self.add(Box::new(UnderScrutinyHandler::new()));
            }
            _ => {}
        }
    }

    pub fn add(&mut self, handler: Box<dyn PrayerHandler>) {
        self.handlers.push(handler);
    }

    /// Java: forName(String pName) — case-insensitive name lookup.
    pub fn for_name(&self, name: &str) -> Option<&dyn PrayerHandler> {
        self.handlers.iter()
            .find(|h| h.get_name().eq_ignore_ascii_case(name))
            .map(|h| h.as_ref())
    }

    /// Java: forPrayer(Prayer) — find handler for this prayer.
    pub fn for_prayer(&self, prayer_name: &str) -> Option<&dyn PrayerHandler> {
        self.handlers.iter()
            .find(|h| h.handles_prayer(prayer_name))
            .map(|h| h.as_ref())
    }

    pub fn len(&self) -> usize { self.handlers.len() }
    pub fn is_empty(&self) -> bool { self.handlers.is_empty() }

    /// Java: PrayerHandlerFactory.deactivatePrayers(GameState, isHomeTurnEnding).
    /// Calls remove_effect() on each active prayer for the given team.
    pub fn deactivate_prayers(game: &mut ffb_model::model::game::Game, is_home: bool) {
        let team_id = if is_home {
            game.team_home.id.clone()
        } else {
            game.team_away.id.clone()
        };
        let prayer_names: Vec<String> = if is_home {
            game.turn_data_home.inducement_set.get_prayers().iter().map(|s| s.to_string()).collect()
        } else {
            game.turn_data_away.inducement_set.get_prayers().iter().map(|s| s.to_string()).collect()
        };
        let mut factory = PrayerHandlerFactory::new();
        factory.initialize(game.rules);
        for prayer_name in prayer_names {
            // std::mem::take extracts prayer_state to avoid double-borrow of game
            let mut prayer_state = std::mem::take(&mut game.prayer_state);
            if let Some(handler) = factory.for_prayer(&prayer_name) {
                handler.remove_effect(&mut prayer_state, game, &team_id);
            }
            game.prayer_state = prayer_state;
        }
    }
}

impl Default for PrayerHandlerFactory {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;

    #[test]
    fn new_factory_is_empty() {
        assert!(PrayerHandlerFactory::new().is_empty());
    }

    #[test]
    fn for_name_miss_returns_none() {
        assert!(PrayerHandlerFactory::new().for_name("Unknown").is_none());
    }

    #[test]
    fn for_prayer_miss_returns_none() {
        assert!(PrayerHandlerFactory::new().for_prayer("blessing").is_none());
    }

    #[test]
    fn initialize_bb2020_registers_sixteen_handlers() {
        let mut f = PrayerHandlerFactory::new();
        f.initialize(Rules::Bb2020);
        assert_eq!(f.len(), 16);
    }

    #[test]
    fn initialize_bb2025_registers_sixteen_handlers() {
        let mut f = PrayerHandlerFactory::new();
        f.initialize(Rules::Bb2025);
        assert_eq!(f.len(), 16);
    }

    #[test]
    fn for_prayer_finds_fouling_frenzy_after_init_bb2020() {
        let mut f = PrayerHandlerFactory::new();
        f.initialize(Rules::Bb2020);
        assert!(f.for_prayer("FOULING_FRENZY").is_some());
    }

    #[test]
    fn for_prayer_finds_treacherous_trapdoor_after_init_bb2025() {
        let mut f = PrayerHandlerFactory::new();
        f.initialize(Rules::Bb2025);
        assert!(f.for_prayer("TREACHEROUS_TRAPDOOR").is_some());
    }

    #[test]
    fn for_prayer_finds_dazzling_catching_after_init_bb2025() {
        let mut f = PrayerHandlerFactory::new();
        f.initialize(Rules::Bb2025);
        assert!(f.for_prayer("DAZZLING_CATCHING").is_some());
    }

    #[test]
    fn bb2020_has_necessary_violence_bb2025_has_dazzling_catching() {
        let mut f2020 = PrayerHandlerFactory::new();
        f2020.initialize(Rules::Bb2020);
        assert!(f2020.for_prayer("NECESSARY_VIOLENCE").is_some());
        assert!(f2020.for_prayer("DAZZLING_CATCHING").is_none());

        let mut f2025 = PrayerHandlerFactory::new();
        f2025.initialize(Rules::Bb2025);
        assert!(f2025.for_prayer("DAZZLING_CATCHING").is_some());
        assert!(f2025.for_prayer("NECESSARY_VIOLENCE").is_none());
    }

    #[test]
    fn initialize_does_not_panic() {
        let mut f = PrayerHandlerFactory::new();
        f.initialize(Rules::Bb2025);
        f.initialize(Rules::Bb2020);
    }

    #[test]
    fn deactivate_prayers_removes_fan_interaction_effect() {
        use crate::step::framework::test_team;
        use ffb_model::model::game::Game;
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2020);
        // Add FAN_INTERACTION prayer to home team inducement set
        game.turn_data_home.inducement_set.add_prayer("FAN_INTERACTION");
        // Set the prayer effect in prayer_state
        game.prayer_state.add_fan_interaction("home");
        assert!(game.prayer_state.has_fan_interaction("home"));
        // Deactivate prayers for home team
        PrayerHandlerFactory::deactivate_prayers(&mut game, true);
        assert!(!game.prayer_state.has_fan_interaction("home"));
    }

    #[test]
    fn deactivate_prayers_no_op_if_no_prayers() {
        use crate::step::framework::test_team;
        use ffb_model::model::game::Game;
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2020);
        // No prayers registered; must not panic
        PrayerHandlerFactory::deactivate_prayers(&mut game, true);
    }
}
